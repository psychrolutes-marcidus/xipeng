use std::hash::{DefaultHasher, Hash, Hasher};

use crate::types::linestringm::LineStringM;
use crate::types::pointm::PointM;

// linestring segmenter goes here
pub fn segment_linestring<const CRS: u64, F>(ls: LineStringM<CRS>, func: F) -> Vec<LineStringM<CRS>>
where
    F: Fn(PointM<CRS>, PointM<CRS>) -> bool,
{
    #[cfg(debug_assertions)]
    let clone = ls.clone();

    let mut ls = ls;
    let mut output = vec![];
    let mut split_idxs: Vec<usize> = vec![];
    for (idx, ele) in ls.0.windows(2).enumerate() {
        if !func(ele[0].into(), ele[1].into()) {
            split_idxs.push(idx + 1);
        }
    }
    for ele in split_idxs {
        output.push(LineStringM(ls.0.drain(..ele).collect::<Vec<_>>())); //? +/- 1?;
    }
    if !ls.0.is_empty() {
        output.push(LineStringM(ls.0.drain(..).collect()));
    }

    if output.is_empty() {
        output.push(ls);
    }

    #[cfg(debug_assertions)]
    debug_assert_eq!(
        clone,
        LineStringM(
            output
                .iter()
                .map(|ls| ls.clone().0)
                .collect::<Vec<_>>()
                .concat()
        )
    );
    debug_assert!(
        output.iter().any(|ls| ls.0.len() != 1),
        "Linestrings may not have length 1"
    );
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::coordm::CoordM;
    use geo::Distance;
    use geographiclib_rs::Geodesic;
    use pretty_assertions::{assert_eq, assert_ne};
    use wkb::reader::read_wkb;
    #[test]
    fn no_segment() {
        let coords: Vec<CoordM<4326>> = [(1.0, 2.0, 0.0), (2.0, 3.0, 1.0), (3.0, 4.0, 2.0)]
            .map(|f| f.into())
            .to_vec();
        let func = |f: PointM, s: PointM| (s.coord.m - f.coord.m) <= 1.1;
        let res = segment_linestring(LineStringM(coords.clone()), func);
        assert_eq!(res[0], LineStringM(coords));
    }

    #[test]
    fn yes_segment() {
        let coords: Vec<CoordM<4326>> = [
            (1.0, 2.0, 0.0),
            (2.0, 3.0, 1.0),
            (3.0, 4.0, 3.0),
            (4.0, 5.0, 4.0),
        ]
        .map(|f| f.into())
        .to_vec();
        let func = |f: PointM, s: PointM| (s.coord.m - f.coord.m) <= 1.1;
        let res = segment_linestring(LineStringM(coords.clone()), func);
        let expected = vec![
            LineStringM::<4326>(
                [(1.0, 2.0, 0.0), (2.0, 3.0, 1.0)]
                    .map(|f| f.into())
                    .to_vec(),
            ),
            LineStringM::<4326>(
                [(3.0, 4.0, 3.0), (4.0, 5.0, 4.0)]
                    .map(|f| f.into())
                    .to_vec(),
            ),
        ];
        assert_eq!(res, expected);
    }

    #[test]
    fn illegal_segment() {
        let coords: Vec<CoordM<4326>> = [
            (1.0, 2.0, 0.0),
            (2.0, 3.0, 1.0),
            (3.0, 4.0, 3.0),
            (4.0, 5.0, 5.0),
            (5.0, 6.0, 6.0),
        ]
        .map(|f| f.into())
        .to_vec();
        let func = |f: PointM, s: PointM| (s.coord.m - f.coord.m) <= 1.1;
        let res = segment_linestring(LineStringM(coords.clone()), func);
        assert!(
            res.iter().any(|ls| ls.0.len() != 1),
            "Linestrings with length ==1 is disallowed"
        );
    }

    #[test]
    fn funny_trajectory() {
        const HEXSTRING: &str = include_str!("./resources/207138000.txt");

        let bytea = hex::decode(HEXSTRING).unwrap();
        let wkb = read_wkb(&bytea).unwrap();
        let lsm = LineStringM::<4326>::try_from(wkb).unwrap();

        let func = |f: PointM, s: PointM| {
            geo::algorithm::line_measures::metric_spaces::Geodesic.distance(f, s) <= 1000.
                || s.coord.m - f.coord.m <= 60.
        };
        dbg!(&lsm.0.len());
        let segments = segment_linestring(lsm, func);
        dbg!(segments.len());
        dbg!(segments);

    }
}
