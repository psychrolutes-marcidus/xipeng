use geo::{Distance, MultiLineString};
use linesonmaps::{
    algo::segmenter::{segmenter, TrajectorySplit},
    types::{linestringm, pointm::PointM},
};
use pgrx::prelude::*;
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

// #[pg_extern]
// fn render_linestring(linestring: &[u8]) {
//     let conv_wkb = reader::read_wkb(linestring).expect("Something");
//     let linestringm: linestringm::LineStringM<4326> =
//         linestringm::LineStringM::try_from(conv_wkb).expect("Something2");
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
