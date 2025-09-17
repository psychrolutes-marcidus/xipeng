use std::hash::{DefaultHasher, Hash, Hasher};

use crate::types::linestringm::LineStringM;
use crate::types::pointm::PointM;

pub fn segment_linestring<const CRS: u64, F>(ls: LineStringM<CRS>, func: F) -> Vec<LineStringM<CRS>>
where
    F: Fn(PointM<CRS>, PointM<CRS>) -> bool,
{
    #[cfg(debug_assertions)]
    let clone = ls.0.clone();

    let mut ls = ls.0;
    // let mut split_idx: usize = 0;
    let mut offset: usize = 0;
    let mut output: Vec<LineStringM<CRS>> = vec![];

    for (idx, ele) in ls.clone().windows(2).enumerate() {
        if !func(ele[0].into(), ele[1].into()) {
            let rest = ls.split_off(idx + 1 - offset);
            offset = idx;
            output.push(LineStringM(ls));
            debug_assert_ne!(
                output.last().map(|l| l.0.len()),
                Some(1),
                "Linestrings cannot have length 1 {0:?}",
                output.last()
            );
            ls = rest;

            #[cfg(debug_assertions)]
            debug_assert_eq!(
                output.iter().map(|l| l.0.len()).sum::<usize>() + ls.len(),
                clone.len()
            );
        }
    }
    if !ls.is_empty() {
        output.push(LineStringM(ls));
    }

    // tests for presence of any illegal linestrings
    let (legal, illegal): (Vec<_>, Vec<_>) =
        output.into_iter().enumerate().partition(|p| p.1.0.len() != 1);


    // merges illegal linestrings with their left neighboring linestring
    let mut output: Vec<LineStringM<CRS>> = vec![];
    for (idx,ele) in legal.into_iter() {
        match illegal.iter().find(|p| idx+1==p.0 ) {
            Some((_,ls)) => {output.push(LineStringM([ele.0,ls.0.clone()].concat())); },
            None => {output.push(ele);},
        }
    }

    #[cfg(debug_assertions)]
    {
        let conc = output
            .iter()
            .map(|l| l.clone().0)
            .collect::<Vec<_>>()
            .concat();
        debug_assert_eq!(clone, conc);
    }

    output
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::types::coordm::CoordM;
    use crate::types::multilinestringm::MultiLineStringM;
    use geo::Distance;
    use geographiclib_rs::Geodesic;
    use hex::encode;
    use pretty_assertions::{assert_eq, assert_ne};
    use wkb::reader::read_wkb;
    use wkb::writer::WriteOptions;
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
        // segments.iter().map(|ls| wkb::writer::write)
        // dbg!(segments);

        let mut output: Vec<u8> = Vec::new();

        let _ = wkb::writer::write_multi_line_string(
            &mut output,
            &MultiLineStringM(segments.clone()),
            &WriteOptions {
                endianness: wkb::Endianness::LittleEndian,
            },
        );

        let hexstring = encode(&output);
        fs::write("multilinestring.txt", hexstring.to_ascii_uppercase()).unwrap();

        // not sure what to test for :))
        assert_eq!(segments.len(),23);
    }
}
