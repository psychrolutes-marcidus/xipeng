use chrono::DateTime;
use data::loaders::database::DbConn;
use dotenvy::*;
use geo::Distance;
use itertools::{self, Itertools};
use linesonmaps::algo::segmenter::{TrajectorySplit, segmenter};
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

    let dist_thres = (1..).into_iter().map(|p| p as f64 * 100_f64);
    let time_thres = (1..).into_iter().map(|p| p as f64 * 20_f64);
    let cartesian = dist_thres
        .take_while(|d| *d <= 2000.)
        .cartesian_product(time_thres.take_while(|t| *t <= 360.)).collect_vec();

    dbg!(cartesian.len());

    println!("Hello, world!");
}

const fn time_dist(first: PointM, second: PointM, thres: f64) -> bool {
    second.coord.m - first.coord.m < thres
}

fn dist(first: PointM, second: PointM, thres: f64) -> bool {
    use geo::algorithm::line_measures::metric_spaces::Geodesic;
    Geodesic.distance(first, second) < thres
}
