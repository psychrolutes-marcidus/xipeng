use std::hash::{DefaultHasher, Hash, Hasher};

use crate::types::linestringm::LineStringM;
use crate::types::pointm::PointM;

// linestring segmenter goes here
pub fn segment_linestring<const CRS: u64, F>(ls: LineStringM<CRS>, func: F) -> Vec<LineStringM<CRS>>
where
    F: Fn(PointM<CRS>, PointM<CRS>) -> bool,
{
    // #[cfg(debug_assertions)]
    // let mut hasher = DefaultHasher::new();
    // #[cfg(debug_assertions)]
    // ls.hash(&mut hasher);
    // #[cfg(debug_assertions)]
    // let hash = hasher.finish();

    #[cfg(debug_assertions)]
    let clone = ls.clone();

    let mut ls = ls;
    let mut output = vec![];
    let mut split_idxs: Vec<usize> = vec![];
    // let mut offset: usize = 0;
    // output.last().unwrap().0.push(*ls.0.first().unwrap());
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

    #[cfg(debug_assertions)] // This is apparently required
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

    // ! DISSALLOW LINESTRINGS WITH LENGTH == 1;

    // let mut lsi = ls.points();

    // while true {
    //     let first = lsi.next();
    //     let second = lsi.next();
    //     match first.and_then(|f| second.map(|s| func(f,s))) {
    //         Some(true) => {output.last().unwrap().0.extend_from_slice(&[first,second]);},
    //         Some(false) => {},
    //         None => {},
    //     }
    // }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::coordm::CoordM;
    use pretty_assertions::{assert_eq, assert_ne};
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
            (5.0,6.0,6.0),
        ]
        .map(|f| f.into())
        .to_vec();
        let func = |f: PointM, s: PointM| (s.coord.m - f.coord.m) <= 1.1;
        let res = segment_linestring(LineStringM(coords.clone()), func);
        // let expected = vec![
        //     LineStringM::<4326>(
        //         [(1.0, 2.0, 0.0), (2.0, 3.0, 1.0)]
        //             .map(|f| f.into())
        //             .to_vec(),
        //     ),
        //     LineStringM::<4326>(
        //         [(3.0, 4.0, 3.0), (4.0, 5.0, 4.0)]
        //             .map(|f| f.into())
        //             .to_vec(),
        //     ),
        // ];
        assert!(res.iter().any(|ls|ls.0.len()!=1),"Linestrings with length ==1 is disallowed");
    }
}
