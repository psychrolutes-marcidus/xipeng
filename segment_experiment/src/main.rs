use chrono::DateTime;
use data::{loaders::database::DbConn, tables::Ships, tables::trajectories::Trajectories};
use dotenvy::*;
use geo_traits::*;
use linesonmaps::algo::segmenter::{TrajectorySplit, segment_linestring, segmenter};
use linesonmaps::types::linestringm::LineStringM;
use linesonmaps::types::pointm::PointM;
use linesonmaps::types::*;
use rayon::prelude::*;

type LineString = LineStringM<4326>;
type MMSI = i32;

// output: segmented linestrings (with MMSI), number of segments, average length of segments all across different time parameters
fn main() {
    dotenv().unwrap();
    let mut conn = DbConn::new().unwrap();

    let from =
        DateTime::parse_from_str("2024-01-01 00:00:00 +0000", "%Y-%m-%d %H:%M:%S%.3f %z").unwrap();
    let to =
        DateTime::parse_from_str("2024-01-02 00:00:00 +0000", "%Y-%m-%d %H:%M:%S%.3f %z").unwrap();

    let crap = conn.fetch_data(from.into(), to.into()).unwrap();

    let linestrings = crap.trajectories;

    let linestrings: Vec<(MMSI, LineString)> = linestrings
        .mmsi
        .into_iter()
        .zip(linestrings.trajectory)
        .filter(|p| p.0.to_string().len() == 9)
        .map(|(mmsi, ls)| (mmsi, LineStringM::<4326>(ls.0.into_iter().collect()))) //TODO: remove
        .take(10)
        .collect();

    let _ = linestrings
        .iter()
        .inspect(|f| {
            assert!(f.1.0.is_sorted_by_key(|k| k.m), "mmsi ={}", f.0);
        })
        .collect::<Vec<_>>();
    dbg!(linestrings.first().unwrap().0);
    // panic!();
    const THRESHOLDS: [f64; 7] = [15., 30., 60., 75., 90., 105., 120.];
    let header = "mmsi, total_len, time_threshold, num_splits, avg_subtraj_len\n";
    let collected = linestrings
        .par_iter()
        .map(|(mmsi, ls)| {
            let c = THRESHOLDS.map(|t| segmenter(ls.clone(), |f, s| time_dist(f, s, t)));
            (mmsi, c)
        })
        .map(|(mmsi, measures)| {
            let total_len = TrajectorySplit::concat_to_linestring(measures[0].clone())
                .unwrap()
                .0
                .len();
            let rows = measures.into_iter().enumerate().map(|(idx, m)| {
                format!(
                    "{mmsi},{total_len},{0},{1},{2}\n",
                    THRESHOLDS[idx],
                    m.len(),
                    TrajectorySplit::concat_to_linestring(m.clone())
                        .unwrap()
                        .0
                        .len()
                        / m.len()
                )
            }).collect::<Vec<_>>().concat();
            rows
        }).collect::<Vec<_>>().concat();
        // .map(|(mmsi, meas)| {
        //     format!(
        //         "MMSI={2}\t time intervals = {3:?} segments created = {0:?}\t average length = {1:?}\t total length = {4}\n",
        //         meas.iter()
        //             .map(|split| split.len())
        //             .collect::<Vec<_>>(),
        //         meas.iter()
        //             .map(|split| split
        //                 .iter()
        //                 // .map(|ls| ls.0.len())
        //                 .map(|s|{match s {
        //                     TrajectorySplit::Point(p) => {1},
        //                     TrajectorySplit::SubTrajectory(sls) => {sls.0.len()}
        //                 }})
        //                 .fold(0, |acc, x| acc + x)
        //                 / split.len())
        //             .collect::<Vec<_>>(),
        //         mmsi
        //     ,THRESHOLDS, TrajectorySplit::concat_to_linestring(meas[0].clone()).unwrap().0.len())
        // })
        // .collect::<Vec<_>>();
    // .concat();
    let p = "segment_experiment_results.csv";
    let collected = format!("{header}{collected}");
    std::fs::write(p, collected.as_str()).unwrap();
    println!("output experiment results to {0}", p);
}

const fn time_dist(first: PointM, second: PointM, thres: f64) -> bool {
    second.coord.m - first.coord.m < thres
}
