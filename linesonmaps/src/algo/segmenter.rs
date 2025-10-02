use std::ops::Sub;

use geo::coord;
use geo_traits::{CoordTrait, GeometryTrait, LineTrait};

use crate::types::linestringm::LineStringM;
use crate::types::multilinestringm::MultiLineStringM;
use crate::types::pointm::PointM;

/// Splits a linestring into (potentially) several sub-segments using a splitting function.
/// 
/// `ls`: The input linestring
/// `func`: A function that compares to subsequent points, The original linestring will be split if the function returns `false`
pub fn segment_linestring<const CRS: u64, F>(ls: LineStringM<CRS>, func: F) ->  MultiLineStringM<CRS>
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

    output.into()
}


pub fn line_to_square<Line:LineTrait + std::fmt::Debug>(line: Line, a: f64, b: f64, c: f64, d: f64) 
where 
    Line: GeometryTrait<T = f64>
{
    let dx = line.end().x() - line.start().x();
    let dy = line.end().y() - line.start().y();
    let dnorm = f64::sqrt(dx*dx+dy*dy);
    let ndx = dx/dnorm;
    let ndy = dy/dnorm;
    let vec = vec![ndx,ndy]; // describes the direction of the line as a normal vector
    let vec_orth_c = vec![-ndy*c, ndx*c]; // vec_orth_c/d project their length orthogonally along the normal vector by the given lengths c,d respectively
    let vec_orth_d = vec![ndy*d, -ndx*d];
    let c_coord = vec![line.start().x()+vec_orth_c.first().unwrap(), line.start().y()+vec_orth_c.last().unwrap()]; // coordinate of left (port) of line AABB
    let d_coord = vec![line.start().x()+vec_orth_d.first().unwrap(), line.start().y()+vec_orth_d.last().unwrap()]; // coordinate of right (startboard) of line AABB
    let c_coord_end = vec![line.end().x()+vec_orth_c.first().unwrap(), line.end().y()+vec_orth_c.last().unwrap()];
    let d_coord_end = vec![line.end().x()+vec_orth_d.first().unwrap(), line.end().y()+vec_orth_d.last().unwrap()];
    dbg!(c_coord);
    dbg!(d_coord);
    dbg!(c_coord_end);
    dbg!(d_coord_end);

    
    
    let coords = &line.coords();

    let slope = line.end().y() - line.start().y()/(line.end().x() - line.start().x());


}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::types::coordm::CoordM;
    use crate::types::linem::LineM;
    use crate::types::multilinestringm::MultiLineStringM;
    use geo::{coord, Distance, Line};
    use hex::encode;
    use pretty_assertions::{assert_eq, assert_ne};
    use wkb::reader::read_wkb;
    use wkb::writer::WriteOptions;

    #[test]
    fn dumb_test() {
        let line = Line::new(coord! { x: 0., y: 0. }, coord! { x: 4., y: 1. });
        /*let coords: Vec<CoordM<4326>> = [(1.0, 2.0, 0.0), (5.0, 3.0, 1.0), (3.0, 4.0, 2.0)]
            .map(|f| f.into())
            .to_vec();
        let first_line = LineM::from((coords[0],coords[1]));
        */

        line_to_square(line, 1.0,1.0,1.0,1.0);
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
        assert_eq!(segments.0.len(),23);
    }
}
