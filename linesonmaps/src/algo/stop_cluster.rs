use chrono::{DateTime, TimeDelta, Utc};
use geo::Distance;
use itertools::*;
use std::collections::HashSet;
use std::num::NonZero;
use typed_builder::TypedBuilder;

use crate::types::pointm::PointM;

const MS_TO_KNOT: f64 = 1.9438400;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Classification {
    Core(usize),
    Edge(usize),
    Noise,
    Unclassified,
}

// impl<Dist, Sog> DbScanBuilder<Dist, Sog>
// where
//     Dist: Fn(&PointM, &PointM) -> f64,
//     Sog: Fn(f64) -> bool,
// {
//     pub fn builder() -> Self {
//         Self {
//             min_cluster_size: None,
//             dist_thres: None,
//             speed_thres: None,
//             max_time_thres: None,
//         }
//     }
// }

#[derive(TypedBuilder, Debug)]
pub struct DbScanConf<Dist, const CRS: u64>
where
    Dist: Fn(&PointM<CRS>, &PointM<CRS>) -> f64,
{
    /// Minimum number of 'nearby' points within `dist_thres` to a [Classification::Core] Point
    pub(crate) min_cluster_size: NonZero<usize>,
    /// Distance from a [Classification::Edge] Point to a [Classification::Core] Point
    pub(crate) dist: Dist,
    /// Maximum distance to a [Classification::Core] point
    dist_thres: f64,
    /// Maximum Speed Over Ground (SOG) for a point to be clustered
    pub(crate) speed_thres: f32,
    /// Maximum time interval before any succeeding points are left out of cluster
    pub(crate) max_time_thres: TimeDelta,
    #[builder(setter(skip),default=Vec::new())]
    classes: Vec<Classification>,
}

impl<Dist, const CRS: u64> DbScanConf<Dist, CRS>
where
    Dist: Fn(&PointM<CRS>, &PointM<CRS>) -> f64,
{
    // inpsired by existing DBSCAN implementation https://docs.rs/dbscan/latest/src/dbscan/lib.rs.html#184-205
    fn expand_custer(
        &mut self,
        queue: &mut Vec<usize>,
        points: &[(PointM<CRS>, f32)],
        cluster_idx: usize,
        dist_thres: f64,
    ) -> bool {
        use Classification::{Core, Edge, Noise, Unclassified};
        let mut new_cluster = false;
        while let Some(i) = queue.pop() {
            let neighbors = self.range_query_hash_sog(&points[i].0, points, dist_thres);

            if neighbors.len() < self.min_cluster_size.get() {
                continue;
            }
            new_cluster = true;
            self.classes[i] = Core(cluster_idx);

            for ele in neighbors.iter().copied() {
                // map noise labels to at least edge
                if matches!(self.classes[ele], Noise) {
                    self.classes[ele] = Edge(cluster_idx);
                }
                if !matches!(self.classes[ele], Unclassified) {
                    continue;
                } else {
                    self.classes[ele] = Noise
                }

                queue.push(ele);
            }
            // debug_assert!(
            //     neighbors
            //         .iter()
            //         .copied()
            //         .all(|p| !matches!(self.classes[p], Unclassified)),
            //     "all neighbor points should be at least labelled at noise"
            // );
        }

        new_cluster
    }

    pub fn run<'p>(
        &mut self,
        points: &'p [(PointM<CRS>, f32)],
    ) -> Vec<(&'p PointM<CRS>, Classification)> {
        use Classification::{Noise, Unclassified};

        self.classes = vec![Unclassified; points.len()];

        let mut cluster = 0_usize;
        let mut queue = Vec::<usize>::new();

        for i in 0..points.len() {
            if !matches!(self.classes[i], Unclassified) {
                continue;
            }
            self.classes[i] = Noise;
            queue.push(i);
            if self.expand_custer(&mut queue, points, cluster, self.dist_thres) {
                cluster += 1;
            }
        }

        let res: Vec<(&'p PointM<CRS>, Classification)> = points
            .iter()
            .map(|(p, _)| p)
            .zip(std::mem::take(&mut self.classes))
            .collect();

        // #[cfg(debug_assertions)]
        // {
        //     debug_assert!(
        //         res.iter()
        //             .sorted_by_cached_key(|(_, c)| match c {
        //                 Core(v) => *v,
        //                 Edge(v) => *v,
        //                 Noise => 0,
        //                 Unclassified => unreachable!(),
        //             })
        //             .collect_vec()
        //             .chunk_by(|(_, a), (_, b)| a == b)
        //             .all(|c| c.iter().any(|(_, p)| matches!(p, Core(_) | Noise))), // either a chunk has a core point, or it is a "noise" cluster
        //         "every cluster should have atleast 1 core point"
        //     );
        // }

        res
        // todo!()
    }

    fn range_query_hash_sog<'p>(
        &self,
        qp: &'p PointM<CRS>,
        points: &'p [(PointM<CRS>, f32)],
        dist_thres: f64,
    ) -> HashSet<usize> {
        // if qp is points[i], and points[n] is not a neighbor, then points[n-1] cannot be as well, same for points[m] and points[m+1] with n<i<m
        let mut neighbors = points // linestrings are ordered, so 'neigbors' will only be subslice of points
            .iter()
            .tuple_windows()
            .enumerate()
            .skip_while(|(_, (f, s))| !self.temporal_sog_close(qp, &f.0, f.1)) // skip early points in
            .filter(|(_, (f, s))| (self.dist)(qp, &f.0) < dist_thres)
            .take_while(|(_, (f, s))| self.temporal_sog_close(qp, &f.0, f.1))
            .map(|(i, _)| i)
            .collect::<HashSet<_>>();

        // special case to test if points.last() is a neighbor
        if let Some(s) = points.get(points.len() - 2..=points.len() - 1)
            && self.temporal_sog_close(qp, &s[0].0, s[0].1)
                && (self.dist)(qp, &s[1].0) < dist_thres
                && qp != &s[1].0
            {
                let _ = neighbors.insert(points.len() - 1);
            };
        neighbors
    }

    fn temporal_sog_close(&self, qp: &PointM<CRS>, f: &PointM<CRS>, sog: f32) -> bool {
        let f_dt = DateTime::<Utc>::from_timestamp_secs(f.coord.m as i64).unwrap();
        let qp_dt = DateTime::<Utc>::from_timestamp_secs(qp.coord.m as i64).unwrap();
        let temporally_close = (f_dt - qp_dt).abs() < self.max_time_thres;

        temporally_close && sog < self.speed_thres
    }

    //TODO: maybe range query should be performed with the help of an r-tree, but that necesitates a points table
    #[deprecated]
    fn range_query<'p>(
        &self,
        qp: &PointM<CRS>,
        points: &'p [PointM<CRS>],
        dist_thres: f64,
    ) -> Vec<(usize, &'p PointM<CRS>)> {
        let mut neighbors: Vec<(usize, &PointM<CRS>)> = Vec::with_capacity(10);
        // TODO: takewhile, if point n is not neighbor to qp, then n+1 is neither (after some threshold, but only time)
        for ele in points.iter().enumerate() {
            if (self.dist)(qp, ele.1) < dist_thres
                && (ele.1.coord.m - qp.coord.m).abs() < self.max_time_thres.as_seconds_f64()
                && !(qp == ele.1)
            //disallow qp in neigbor set
            //TODO: and speed
            {
                neighbors.push(ele);
            }
        }

        neighbors
    }
}

#[cfg(test)]
pub mod test {
    use chrono::TimeDelta;
    use geo::{Distance, Euclidean, Geodesic};
    use itertools::Itertools;

    use super::Classification::*;
    use crate::algo::stop_cluster::DbScanConf;
    use crate::types::pointm::PointM;

    #[test]
    fn build_conf() {
        let conf = DbScanConf::builder()
            .dist(|a: &PointM<3857>, b| 0.0)
            .max_time_thres(TimeDelta::zero())
            .speed_thres(1.5)
            .min_cluster_size(42.try_into().unwrap())
            .dist_thres(1.0)
            .build();

        assert_eq!(conf.min_cluster_size, 42.try_into().unwrap());
        assert_eq!(conf.max_time_thres, TimeDelta::zero());
        assert_eq!(conf.speed_thres, 1.5);
    }

    #[test]
    fn simple_cluster_fr_fr() {
        let mut conf = DbScanConf::builder()
            // .dist(|a, b| Geodesic.distance(*a, *b))
            .dist(|a, b| ((b.coord.x - b.coord.x).powi(2) + (b.coord.y - a.coord.y).powi(2)).sqrt())
            .max_time_thres(TimeDelta::new(10, 0).unwrap())
            .min_cluster_size(3.try_into().unwrap())
            .speed_thres(20.0)
            .dist_thres(1.1)
            .build();

        let inputs = [
            (1.5, 2.2),
            (1.0, 1.1),
            (1.2, 1.4),
            (0.8, 1.0),
            (3.7, 4.0),
            (3.9, 3.9),
            (3.6, 4.1),
            (10.0, 100.0),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (f, s))| (PointM::<4326>::from((f, s, i as f64 * 1.0)), 1_f32))
        .collect::<Vec<_>>();

        let clusters = conf.run(&inputs);
        dbg!(&clusters);
        assert_eq!(1, clusters.iter().filter(|å| matches!(å.1, Noise)).count());

        let c = clusters.into_iter().map(|(_, c)| c).collect_vec();

        assert_eq!(
            c,
            vec![
                Edge(0),
                Core(0),
                Core(0),
                Core(0),
                Core(1),
                Core(1),
                Core(1),
                Noise
            ]
        );
    }
}
