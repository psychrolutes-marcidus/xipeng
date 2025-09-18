use geo_traits::*;
use linesonmaps::algo::segmenter::segment_linestring;
use linesonmaps::types::linestringm::LineStringM;
use linesonmaps::types::pointm::PointM;
use linesonmaps::types::*;
use rayon::prelude::*;

type LineString = LineStringM<4326>;

fn main() {
    let linestrings: Vec<LineString> = vec![];

    const THRESHOLDS: [f64; 6] = [5., 10., 15., 30., 60., 120.];
    let collected = linestrings
        .par_iter()
        .map(|ls| {
            let c = THRESHOLDS.map(|t| segment_linestring(ls.clone(), |f, s| time_dist(f, s, t)));
            c
        })
        .inspect(|measures| {
            println!(
                "segments created = {0:?}",
                measures.iter().map(|mls| mls.num_line_strings())
            )
        })
        .collect::<Vec<_>>();

    println!("Hello, world!");
}

const fn time_dist(first: PointM, second: PointM, thres: f64) -> bool {
    second.coord.m - first.coord.m < thres
}
