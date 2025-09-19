use geo_traits::*;
use linesonmaps::algo::segmenter::segment_linestring;
use linesonmaps::types::linestringm::LineStringM;
use linesonmaps::types::pointm::PointM;
use linesonmaps::types::*;
use rayon::prelude::*;

type LineString = LineStringM<4326>;
type MMSI = i32;

// output: segmented linestrings (with MMSI), number of segments, average length of segments all across different time parameters
fn main() {
    let linestrings: Vec<(MMSI, LineString)> = vec![];

    //TODO get trajectories, sorted by length and/or number of points

    const THRESHOLDS: [f64; 6] = [5., 10., 15., 30., 60., 120.];
    let collected = linestrings
        .par_iter()
        .map(|(mmsi, ls)| {
            let c = THRESHOLDS.map(|t| segment_linestring(ls.clone(), |f, s| time_dist(f, s, t)));
            (mmsi, c)
        })
        // .inspect(|measures| {
        //     println!(
        //         "segments created = {0:?}\t average length = {1:?}",
        //         measures
        //             .iter()
        //             .map(|mls| mls.num_line_strings())
        //             .collect::<Vec<_>>(),
        //         measures
        //             .iter()
        //             .map(|mls| mls
        //                 .line_strings()
        //                 .map(|ls| ls.0.len())
        //                 .fold(0, |acc, x| acc + x)
        //                 / mls.num_line_strings())
        //             .collect::<Vec<_>>()
        //     )
        // })
        .map(|(mmsi, meas)| {
            format!(
                "MMSI={2}\t segments created = {0:?}\t average length = {1:?}\n",
                meas.iter()
                    .map(|mls| mls.num_line_strings())
                    .collect::<Vec<_>>(),
                meas.iter()
                    .map(|mls| mls
                        .line_strings()
                        .map(|ls| ls.0.len())
                        .fold(0, |acc, x| acc + x)
                        / mls.num_line_strings())
                    .collect::<Vec<_>>(),
                mmsi
            )
        })
        .collect::<Vec<_>>()
        .concat();
    let p = "segment_experiment_results.txt";
    std::fs::write(p, collected.as_str()).unwrap();
    println!("output experiment results to {0}", p);
}

const fn time_dist(first: PointM, second: PointM, thres: f64) -> bool {
    second.coord.m - first.coord.m < thres
}
