use chrono::{DateTime, TimeDelta, Utc};
use wkb::writer::{WriteOptions, write_line_string, write_point};

use crate::types::coordm::CoordM;
use crate::types::linestringm::LineStringM;
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

    pub fn to_wkb(&self) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        match self {
            Self::SubTrajectory(ls) => {
                let _ = write_line_string(
                    &mut writer,
                    ls,
                    &WriteOptions {
                        endianness: wkb::Endianness::LittleEndian,
                    },
                )
                .expect("failed to write geometry");
            }
            Self::Point(p) => {
                let _ = write_point(
                    &mut writer,
                    p,
                    &WriteOptions {
                        endianness: wkb::Endianness::LittleEndian,
                    },
                )
                .expect("failed to write geometry");
            }
        };
        writer
    }
}

type Split<const CRS: u64> = Vec<TrajectorySplit<CRS>>;

/// Splits a linestring into (potentially) several sub-segments using a splitting function.
///
/// `ls`: The input linestring
/// `func`: A function that compares to subsequent points, The original linestring will be split if the function returns `false`
pub fn segmenter<const CRS: u64, F>(ls: LineStringM<CRS>, func: F) -> Split<CRS>
where
    F: Fn(PointM<CRS>, PointM<CRS>) -> bool,
{
    #[cfg(debug_assertions)]
    let clone = ls.clone();

    let ls = ls.0;
    let mut output: Vec<Vec<CoordM<CRS>>> = vec![vec![
        *ls.first().expect("input trajectory should be nonempty"),
    ]];

    for ele in ls.windows(2) {
        let len = output.len();
        match func(ele[0].into(), ele[1].into()) {
            true => output
                .get_mut(len - 1)
                .unwrap()
                .push(*ele.last().expect("should have exactly 2 elements")),
            false => {
                output.push(vec![*ele.last().unwrap()]);
            }
        }
    }

    // partition based on sub-trajectory length (length ==1 are not "proper" trajectories)
    let splits = output
        .into_iter()
        .map(|v| match v {
            vec if vec.len() == 1 => TrajectorySplit::Point(
                vec.first()
                    .expect("vector should contain exactly 1 point")
                    .into(),
            ),
            otherwise => TrajectorySplit::SubTrajectory(
                LineStringM::new(otherwise)
                    .expect("valid input trajectory implies valid subtrajectory"),
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
        //no points should be removed, and order should be maintained
        let ls = TrajectorySplit::concat_to_linestring(splits.clone()).unwrap();
        debug_assert_eq!(
            ls, clone,
            "linestring segmenter erroneously dropped points and/or changed point ordering"
        );
    }

    splits
}

pub fn segment_timestamp<const CRS: u64, F>(
    ls: LineStringM<CRS>,
    func: F,
) -> Vec<(DateTime<Utc>, TimeDelta)>
where
    F: Fn(PointM<CRS>, PointM<CRS>) -> bool,
{
    let segments = segmenter(ls, func);

    let times = segments
        .into_iter()
        .map(|ts| match ts {
            TrajectorySplit::Point(p) => Some((
                DateTime::from_timestamp_secs(p.coord.m as i64)?,
                TimeDelta::zero(),
            )), // casting to integer will truncate/round down, this is probably fine for creating a start interval
            TrajectorySplit::SubTrajectory(ls) => {
                let first = DateTime::from_timestamp_secs(ls.0.first()?.m as i64)?; // Linestrings generated from `segmenter` always have length > 1, so there should be some points
                const {
                    // quick and dirty testing suggests a too large timestamp is somewhere between 2^42 and 2^43 (i.e. 141338-07-19 02:25:04+00 and 280707-02-04 04:50:08+00), i would be shocked if GST still uses this program by then
                    assert!(DateTime::from_timestamp_secs(1 << 43).is_none());
                    assert!(DateTime::from_timestamp_secs(1 << 42).is_some());
                }
                let last = DateTime::from_timestamp_secs(ls.0.last()?.m.ceil() as i64)?; // calling ceil causes some loss in precision, but ensures last point is included in interval, 
                Some((first, last - first))
            }
        })
        .collect::<Option<Vec<_>>>()
        .expect("failed to convert measure value to DateTime object, measure value may be too big");

    debug_assert!(
        times
            .clone()
            .windows(2)
            .all(|p| ((p[1].0 + p[1].1) - (p[0].0 + p[0].1)).num_milliseconds() <= 1000),
        "time intervals should be non-overlapping (within a threshold)"
    );

    times
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::coordm::CoordM;
    use geo::Distance;
    use pretty_assertions::{assert_eq, assert_ne};
    use wkb::reader::read_wkb;

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
        assert_eq!(
            res.next(),
            Some(TrajectorySplit::SubTrajectory(expected[0].clone()))
        );
        assert_eq!(
            res.next(),
            Some(TrajectorySplit::SubTrajectory(expected[1].clone()))
        );
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
        assert_eq!(
            TrajectorySplit::concat_to_linestring(res).unwrap().0.len(),
            lsm.0.len()
        );
    }
}
