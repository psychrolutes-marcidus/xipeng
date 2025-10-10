use chrono::DateTime;
use data::loaders::database::DbConn;
use data::loaders::database::TrajectoryIter;
use data::loaders::database::insert_sub_traj_inteval;
use dotenvy::dotenv;
use geo::Distance;
use itertools::{self, Itertools};
use linesonmaps::algo::segmenter::{TrajectorySplit, segment_timestamp, segmenter};
use linesonmaps::types::linestringm::LineStringM;
use linesonmaps::types::pointm::PointM;
use rayon::prelude::*;

type LineString = LineStringM<4326>;
#[allow(clippy::upper_case_acronyms)]
type MMSI = i32;

// dist threshold init 50m, step-size: ¿50m?
// time threshold init 10s, step-size: ¿20s?

fn main() {
    dotenv().expect("failed to load environment variables");

    // let dist_thres = (1..).into_iter().map(|p| p as f64 * 100_f64);
    // let time_thres = (1..).into_iter().map(|p| p as f64 * 20_f64);
    // let cartesian = dist_thres
    //     .take_while(|d| *d <= 2000.)
    //     .cartesian_product(time_thres.take_while(|t| *t <= 360.)).collect_vec();

    // dbg!(cartesian.len());
    let mut conn = DbConn::new().expect("failed to establish database connection");
    let func = |f, l| dist(f, l, 1000_f64) && time_dist(f, l, 60_f64);
    let it =
        TrajectoryIter::<500>::new(DbConn::new().expect("failed to establish database connection"))
            .expect("failed to create select iterator");

    let _ = it
        .map(|ts| {
            let commit_res = ts.map(|t| {
                let z = t
                    .mmsi
                    .into_iter()
                    .zip_eq(t.trajectory)
                    .map(|(mmsi, traj)| (mmsi, segment_timestamp(traj, func)))
                    .collect_vec();
                let t = insert_sub_traj_inteval(&mut conn.conn, z)
                    .expect("database error")
                    .commit();
                t
            });
            commit_res
        })
        .flatten_ok()
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to process trajectories");
    println!("Hello, world!");
}

const fn time_dist(first: PointM, second: PointM, thres: f64) -> bool {
    second.coord.m - first.coord.m < thres
}

fn dist(first: PointM, second: PointM, thres: f64) -> bool {
    use geo::algorithm::line_measures::metric_spaces::Geodesic;
    Geodesic.distance(first, second) < thres
}
