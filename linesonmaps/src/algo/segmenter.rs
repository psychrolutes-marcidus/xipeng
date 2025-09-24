use crate::types::coordm::CoordM;
use crate::types::linestringm::LineStringM;
use crate::types::multilinestringm::MultiLineStringM;
use crate::types::pointm::PointM;

#[derive(Debug, Clone, PartialEq)]
pub enum TrajectorySplit<const CRS: u64> {
    /// A split that resulted in a [`LineStringM`]
    SubTrajectory(LineStringM<CRS>),
    /// a split that resulted in a single [`PointM`]
    Point(PointM<CRS>),
}

impl<const CRS: u64> TrajectorySplit<CRS> {
    pub fn concat_to_linestring(splits: Vec<TrajectorySplit<CRS>>) -> Option<LineStringM<CRS>> {
        let concat = splits
            .iter()
            .cloned()
            .map(|ts| match ts {
                TrajectorySplit::Point(p) => {
                    vec![p.coord]
                }
                TrajectorySplit::SubTrajectory(sls) => sls.0,
            })
            .collect::<Vec<_>>()
            .concat();

        LineStringM::new(concat)
    }
}

type Split<const CRS: u64> = Vec<TrajectorySplit<CRS>>;

pub fn segmenter<const CRS: u64, F>(ls: LineStringM<CRS>, func: F) -> Split<CRS>
where
    F: Fn(PointM<CRS>, PointM<CRS>) -> bool,
{
    #[cfg(debug_assertions)]
    let clone = ls.clone();

    let ls = ls.0;
    // let mut offset = 0 as usize;
    let mut output: Vec<Vec<CoordM<CRS>>> = vec![vec![
        *ls.first().expect("input trajectory should be nonempty"),
    ]];

    for (idx, ele) in ls.windows(2).enumerate() {
        // let mut current: Vec<PointM<CRS>> = vec![];
        let len = output.len();
        match func(ele[0].into(), ele[1].into()) {
            true => output
                .get_mut(len - 1)
                .unwrap()
                .push(*ele.last().expect("should have exactly 2 elements")),
            false => {
                output.push(vec![*ele.last().unwrap()]);
            }
        } //TODO: remember to push last element
    }
    let splits = output
        .into_iter()
        .map(|v| match v {
            vec if vec.len() == 1 => TrajectorySplit::Point(
                vec.first()
                    .expect("vector should contain exactly 1 point")
                    .into(),
            ),
            otherwise => TrajectorySplit::SubTrajectory(
                LineStringM::new(otherwise).expect("hejj"),
            ),
        })
        .collect::<Vec<_>>();

    debug_assert!(splits.iter().all(|p| match p {
        TrajectorySplit::SubTrajectory(sls) => {
            sls.0.len() > 1
        }
        _ => true,
    }));

    #[cfg(debug_assertions)]
    {
        let ls = TrajectorySplit::concat_to_linestring(splits.clone()).unwrap();
        debug_assert_eq!(
            ls, clone,
            "linestring segmenter erroneously dropped points and/or changed point ordering"
        );
    }

    splits
}

/// Splits a linestring into (potentially) several sub-segments using a splitting function.
///
/// `ls`: The input linestring
/// `func`: A function that compares to subsequent points, The original linestring will be split if the function returns `false`
#[deprecated]
pub fn segment_linestring<const CRS: u64, F>(ls: LineStringM<CRS>, func: F) -> MultiLineStringM<CRS>
where
    F: Fn(PointM<CRS>, PointM<CRS>) -> bool,
{
    #[cfg(debug_assertions)]
    let clone = ls.0.clone();

    let mut ls = ls.0;
    let mut offset: usize = 0;
    let mut output: Vec<LineStringM<CRS>> = vec![];

    for (idx, ele) in ls.clone().windows(2).enumerate() {
        if !func(ele[0].into(), ele[1].into()) {
            let rest = ls.split_off(idx + 1 - offset);
            offset = idx + 1;
            output.push(LineStringM(ls));
            // debug_assert_ne!(
            //     output.last().map(|l| l.0.len()),
            //     Some(1),
            //     "Linestrings cannot have length 1 {0:?}",
            //     output.last()
            // );
            ls = rest;

            #[cfg(debug_assertions)]
            {
                pretty_assertions::assert_eq!(
                    output.iter().map(|l| l.0.len()).sum::<usize>() + ls.len(),
                    clone.len(),
                );
            }
        }
    }
    if !ls.is_empty() {
        output.push(LineStringM(ls));
    }

    // tests for presence of any illegal linestrings
    let (legal, illegal): (Vec<_>, Vec<_>) = output
        .into_iter()
        .filter(|p| p.0.len() != 0)
        .enumerate()
        .partition(|p| p.1.0.len() != 1); //TODO no length ==0

    // merges illegal linestrings with their left neighboring linestring
    let mut output: Vec<LineStringM<CRS>> = vec![];
    for (idx, ele) in legal.into_iter() {
        match illegal.iter().find(|p| idx + 1 == p.0) {
            Some((_, ls)) => {
                output.push(LineStringM([ele.0, ls.0.clone()].concat()));
            }
            None => {
                output.push(ele);
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        let conc = output
            .iter()
            .map(|l| l.clone().0)
            .collect::<Vec<_>>()
            .concat();
        pretty_assertions::assert_eq!(
            clone,
            conc,
            "Linestring segments erroneously discarded points (original length = {0}, new = {1} subsegments = {2})",
            clone.len(),
            conc.len(),
            output.len(),
        );
    }

    output.into()
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::fs;

    use super::*;
    use crate::types::coordm::CoordM;
    use crate::types::multilinestringm::MultiLineStringM;
    use geo::Distance;
    use hex::encode;
    use pretty_assertions::{assert_eq, assert_ne};
    use wkb::reader::read_wkb;
    use wkb::writer::WriteOptions;

    #[test]
    fn no_segment_segmenter() {
        let coords: Vec<CoordM<4326>> = [(1.0, 2.0, 0.0), (2.0, 3.0, 1.0), (3.0, 4.0, 2.0)]
            .map(|f| f.into())
            .to_vec();
        let func = |f: PointM, s: PointM| (s.coord.m - f.coord.m) <= 1.1;
        let res = segmenter(LineStringM::new(coords.clone()).unwrap(), func);
        assert_eq!(
            TrajectorySplit::concat_to_linestring(res).unwrap().0.len(),
            coords.len()
        );
    }

    #[test]
    fn yes_segment_segmenter() {
        let coords: Vec<CoordM<4326>> = [
            (1.0, 2.0, 0.0),
            (2.0, 3.0, 1.0),
            (3.0, 4.0, 3.0),
            (4.0, 5.0, 4.0),
        ]
        .map(|f| f.into())
        .to_vec();
        let func = |f: PointM, s: PointM| (s.coord.m - f.coord.m) <= 1.1;
        let mut res = segmenter(LineStringM::new(coords.clone()).unwrap(), func).into_iter();

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
        assert_eq!(res.next(),Some(TrajectorySplit::SubTrajectory(expected[0].clone())));
        assert_eq!(res.next(),Some(TrajectorySplit::SubTrajectory(expected[1].clone())));

    }

    #[test]
    fn funny_trajectory_segmenter() {
        const HEXSTRING: &str = include_str!("./resources/207138000.txt");

        let bytea = hex::decode(HEXSTRING).unwrap();
        let wkb = read_wkb(&bytea).unwrap();
        let lsm = LineStringM::<4326>::try_from(wkb).unwrap();
        let lsm = LineStringM::new(lsm.0).unwrap();
        let mut lsm_s = lsm.clone();
        // lsm_s.0.sort_by(|a,b| a.m.total_cmp(&b.m));
        // assert_eq!(lsm,lsm_s);

        let func = |f: PointM, s: PointM| {
            geo::algorithm::line_measures::metric_spaces::Geodesic.distance(f, s) <= 1000.
                || s.coord.m - f.coord.m <= 60.
        };

        let res = segmenter(lsm.clone(), func);
        assert_eq!(TrajectorySplit::concat_to_linestring(res).unwrap().0.len(),lsm.0.len());
    }
    #[test]
    fn no_segment() {
        let coords: Vec<CoordM<4326>> = [(1.0, 2.0, 0.0), (2.0, 3.0, 1.0), (3.0, 4.0, 2.0)]
            .map(|f| f.into())
            .to_vec();
        let func = |f: PointM, s: PointM| (s.coord.m - f.coord.m) <= 1.1;
        let res = segment_linestring(LineStringM(coords.clone()), func);
        assert_eq!(res.0[0], LineStringM(coords));
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
        assert_eq!(res.0, expected);
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
            res.0.iter().any(|ls| ls.0.len() != 1),
            "Linestrings with length ==1 is disallowed"
        );
    }

    #[test]
    #[ignore = "din far"]
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
        dbg!(segments.0.len());

        let mut output: Vec<u8> = Vec::new();

        let _ = wkb::writer::write_multi_line_string(
            &mut output,
            &MultiLineStringM(segments.0.clone()),
            &WriteOptions {
                endianness: wkb::Endianness::LittleEndian,
            },
        );

        let hexstring = encode(&output);
        // fs::write("multilinestring.txt", hexstring.to_ascii_uppercase()).unwrap();

        // not sure what to test for :))
        assert_eq!(segments.0.len(), 23);
    }
}
