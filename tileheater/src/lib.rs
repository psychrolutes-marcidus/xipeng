use std::fmt::Debug;

use geo::{Distance, MultiLineString};
use linesonmaps::{
    algo::segmenter::{segmenter, TrajectorySplit},
    types::{linestringm, pointm::PointM},
};
use pgrx::{pg_sys::bytea, prelude::*};
use wkb::{reader, writer};

::pgrx::pg_module_magic!(name, version);

#[pg_extern]
fn hello_tileheater() -> &'static str {
    "Hello, tileheater"
}

#[pg_extern]
fn segment_linestring(linestring: &[u8], single_points: bool) -> Vec<u8> {
    let conv_wkb = reader::read_wkb(linestring).expect("Something");
    let linestringm: linestringm::LineStringM<4326> =
        linestringm::LineStringM::try_from(conv_wkb).expect("Something2");
    let func = |f, l| dist(f, l, 1000_f64) && time_dist(f, l, 60_f64);
    let splitted = segmenter(linestringm, func);

    match single_points {
        true => {
            let sub_points: Vec<_> = splitted
                .into_iter()
                .map(|x| match x {
                    TrajectorySplit::SubTrajectory(_) => None,
                    TrajectorySplit::Point(point_m) => return Some(point_m),
                })
                .flatten()
                .collect();
            linesonmaps::types::pointm
        }
        false => {
            let sub_traj: Vec<_> = splitted
                .into_iter()
                .map(|x| match x {
                    TrajectorySplit::SubTrajectory(line_string_m) => return Some(line_string_m),
                    TrajectorySplit::Point(_) => None,
                })
                .flatten()
                .collect();
            let multi = linesonmaps::types::multilinestringm::MultiLineStringM::from(sub_traj);
        }
    }
    let mut buf: Vec<u8> = Vec::new();
    let options = wkb::writer::WriteOptions {
        endianness: wkb::Endianness::LittleEndian,
    };
    writer::write_multi_line_string(&mut buf, &multi, &options).expect("Expected a joke");

    buf
}

const fn time_dist(first: PointM, second: PointM, thres: f64) -> bool {
    second.coord.m - first.coord.m < thres
}

fn dist(first: PointM, second: PointM, thres: f64) -> bool {
    use geo::algorithm::line_measures::metric_spaces::Geodesic;
    Geodesic.distance(first, second) < thres
}

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
