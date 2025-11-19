use chrono::{Datelike, TimeDelta, Timelike};
use geo::{Distance, Geodesic, MultiLineString};
use linesonmaps::{
    algo::{
        segmenter::{segmenter, TrajectorySplit},
        stop_cluster::{cluster_to_traj_with_stop_object, DbScanConf},
    },
    types::{linestringm, pointm::PointM},
};
use pgrx::{
    pg_sys::{IndexAttrBitmapKind_INDEX_ATTR_BITMAP_IDENTITY_KEY, TZ_STRLEN_MAX},
    prelude::*,
    AnyArray,
};
use serde::{Deserialize, Serialize};
use wkb::{reader, writer};

pub mod types;

::pgrx::pg_module_magic!(name, version);

#[pg_extern]
fn hello_tileheater() -> &'static str {
    "Hello, tileheater"
}

#[pg_extern(parallel_safe, immutable)]
fn segment_linestring(linestring: &[u8]) -> SetOfIterator<'static, Vec<u8>> {
    let conv_wkb = reader::read_wkb(linestring).expect("Something");
    let linestringm: linestringm::LineStringM<4326> =
        linestringm::LineStringM::try_from(conv_wkb).expect("Something2");
    let func = |f, l| dist(f, l, 1000_f64) && time_dist(f, l, 60_f64);
    let splitted = segmenter(linestringm, func);

    let sub_traj = splitted
        .into_iter()
        .map(|x| match x {
            TrajectorySplit::SubTrajectory(line_string_m) => return Some(line_string_m),
            TrajectorySplit::Point(_) => None,
        })
        .flatten()
        .map(|line| {
            let mut buf: Vec<u8> = Vec::new();
            let options = wkb::writer::WriteOptions {
                endianness: wkb::Endianness::LittleEndian,
            };
            writer::write_line_string(&mut buf, &line, &options).expect("Nothing");

            buf
        });

    SetOfIterator::new(sub_traj)
}

const fn time_dist(first: PointM, second: PointM, thres: f64) -> bool {
    second.coord.m - first.coord.m < thres
}

fn dist(first: PointM, second: PointM, thres: f64) -> bool {
    use geo::algorithm::line_measures::metric_spaces::Geodesic;
    Geodesic.distance(first, second) < thres
}

#[pg_extern(parallel_safe, immutable)]
fn segment_points(linestring: &[u8]) -> SetOfIterator<'static, Vec<u8>> {
    let conv_wkb = reader::read_wkb(linestring).expect("Something");
    let linestringm: linestringm::LineStringM<4326> =
        linestringm::LineStringM::try_from(conv_wkb).expect("Something2");
    let func = |f, l| dist(f, l, 1000_f64) && time_dist(f, l, 60_f64);
    let splitted = segmenter(linestringm, func);

    let sub_traj: Vec<_> = splitted
        .into_iter()
        .map(|x| match x {
            TrajectorySplit::SubTrajectory(_) => None,
            TrajectorySplit::Point(point_m) => return Some(point_m),
        })
        .flatten()
        .collect();

    let data = sub_traj.into_iter().map(|x| {
        let mut buf: Vec<u8> = Vec::new();
        let options = wkb::writer::WriteOptions {
            endianness: wkb::Endianness::LittleEndian,
        };
        writer::write_point(&mut buf, &x, &options).expect("Nothing");

        buf
    });

    SetOfIterator::new(data)
}

#[derive(Clone, Default, PostgresType, Serialize, Deserialize, AggregateName)]
#[pg_binary_protocol]
pub struct StopObjectAgg {
    data: Vec<(Vec<u8>, Option<f64>)>,
}

#[derive(Clone, PostgresType, Serialize, Deserialize, AggregateName)]
#[pg_binary_protocol]
pub struct StopObject {
    geom: Vec<u8>,
    time_start: TimestampWithTimeZone,
    time_end: TimestampWithTimeZone,
}

#[pg_aggregate(parallel_safe)]
impl Aggregate<StopObjectAgg> for StopObjectAgg {
    const INITIAL_CONDITION: Option<&'static str> = Some(r#"{ "data": []}"#);
    type Args = (Vec<u8>, Option<f64>);
    type Finalize = Vec<StopObject>;
    fn state(
        mut current: Self::State,
        arg: Self::Args,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
        current.data.push(arg);
        current
    }
    fn combine(
        mut current: Self::State,
        other: Self::State,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
        current.data.extend_from_slice(&other.data);
        current
    }
    fn finalize(
        current: Self::State,
        _direct_args: Self::OrderedSetArgs,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::Finalize {
        let mut points: Vec<_> = current
            .data
            .iter()
            .map(|(p, t)| {
                let point = wkb::reader::read_wkb(p).expect("Something");
                (
                    PointM::<4326>::try_from(point).expect("Something2"),
                    match t {
                        Some(t) => *t as f32,
                        None => f32::NAN, // This is instead of an option :)
                    },
                )
            })
            .collect();
        points.sort_by(|a, b| a.0.coord.m.total_cmp(&b.0.coord.m));

        let mut conf = DbScanConf::builder()
            .dist(|a: &PointM<4326>, b: &PointM<4326>| Geodesic.distance(*a, *b))
            .max_time_thres(TimeDelta::new(30 * 60, 0).expect("This did not work"))
            .speed_thres(1.5)
            .min_cluster_size(150.try_into().expect("Neither did this"))
            .dist_thres(250.0)
            .build();
        let clusters = conf.run(&points);

        cluster_to_traj_with_stop_object(clusters)
            .0
            .iter()
            .filter_map(|a| match a {
                linesonmaps::algo::stop_cluster::StopOrLs::Stop { polygon, tz_tange } => {
                    let mut buf: Vec<u8> = Vec::new();
                    let options = wkb::writer::WriteOptions {
                        endianness: wkb::Endianness::LittleEndian,
                    };
                    let ts = TimestampWithTimeZone::new(
                        tz_tange.0.year() as i32,
                        tz_tange.0.month() as u8,
                        tz_tange.0.day() as u8,
                        tz_tange.0.hour() as u8,
                        tz_tange.0.minute() as u8,
                        tz_tange.0.second() as f64,
                    )
                    .unwrap();
                    let te = TimestampWithTimeZone::new(
                        tz_tange.1.year() as i32,
                        tz_tange.1.month() as u8,
                        tz_tange.1.day() as u8,
                        tz_tange.1.hour() as u8,
                        tz_tange.1.minute() as u8,
                        tz_tange.1.second() as f64,
                    )
                    .unwrap();

                    wkb::writer::write_polygon(&mut buf, &polygon, &options)
                        .expect("Something else");
                    Some(StopObject {
                        geom: buf,
                        time_start: ts,
                        time_end: te,
                    })
                }
                _ => None,
            })
            .collect()
    }
}

// #[pg_extern(parallel_safe, immutable)]
// fn extract_stop_object(
//     points: AnyArray,
// ) -> TableIterator<
//     'static,
//     (
//         name!(geom, Vec<u8>),
//         name!(time_start, TimestampWithTimeZone),
//         name!(time_end, TimestampWithTimeZone),
//     ),
// > {
//     let points: Vec<(Option<Vec<u8>>, Option<f64>)> = points
//         .into_iter()
//         .flatten()
//         .map(|x| {
//             let a = x.into::<(Option<Vec<u8>>, Option<f64>)>();

//         })
//         .collect();

// let (points, sog): (Vec<Option<Vec<u8>>>, Vec<Option<f64>>) = points.iter().map(|&x| x).unzip();

// let points: Vec<_> = points
//     .iter()
//     .flatten()
//     .map(|x| {
//         let conv_wkb = reader::read_wkb(x).expect("Something");
//         PointM::try_from(conv_wkb).expect("Something2")
//     })
//     .zip(sog.iter().map(|x| x.map(|x| x as f32)).map(|x| match x {
//         Some(s) => s,
//         None => f32::NAN,
//     }))
//     .collect();

// let mut conf = DbScanConf::builder()
//     .dist(|a: &PointM<4326>, b: &PointM<4326>| Geodesic.distance(*a, *b))
//     .max_time_thres(TimeDelta::new(30 * 60, 0).unwrap())
//     .speed_thres(1.5)
//     .min_cluster_size(10.try_into().unwrap())
//     .dist_thres(250.0)
//     .build();

// let clusters = conf.run(&points);

// let clusters: Vec<_> = cluster_to_traj_with_stop_object(clusters)
//     .0
//     .into_iter()
//     .filter_map(|x| match x {
//         linesonmaps::algo::stop_cluster::StopOrLs::Stop { polygon, tz_tange } => {
//             Some((polygon, tz_tange.0, tz_tange.1))
//         }
//         _ => None,
//     })
//     .map(|(p, ts, te)| {
//         let mut buf: Vec<u8> = Vec::new();
//         let options = wkb::writer::WriteOptions {
//             endianness: wkb::Endianness::LittleEndian,
//         };
//         writer::write_polygon(&mut buf, &p, &options).unwrap();
//         let ts = TimestampWithTimeZone::new(
//             ts.year() as i32,
//             ts.month() as u8,
//             ts.day() as u8,
//             ts.hour() as u8,
//             ts.minute() as u8,
//             ts.second() as f64,
//         )
//         .unwrap();
//         let te = TimestampWithTimeZone::new(
//             te.year() as i32,
//             te.month() as u8,
//             te.day() as u8,
//             te.hour() as u8,
//             te.minute() as u8,
//             te.second() as f64,
//         )
//         .unwrap();
//         (buf, ts, te)
//     })
//     .collect();

// TableIterator::new(clusters)
//     todo!()
// }
#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_hello_tileheater() {
        assert_eq!("Hello, tileheater", crate::hello_tileheater());
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
