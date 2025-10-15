use chrono::TimeDelta;
use std::collections::HashMap;
use std::num::NonZero;
use typed_builder::TypedBuilder;

use crate::types::pointm::PointM;

pub enum Classification<'p> {
    Core(&'p PointM),
    Edge(&'p PointM),
    Unclassified(&'p PointM),
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
pub struct DbScanConf<Dist>
where
    Dist: Fn(&PointM, &PointM) -> f64,
{
    /// Minimum number of 'nearby' points within `dist_thres` to a [Classification::Core] Point
    pub(crate) min_cluster_size: NonZero<u64>,
    /// Distance from a [Classification::Edge] Point to a [Classification::Core] Point
    pub(crate) dist: Dist,
    /// Maximum Speed Over Ground (SOG) for a point to be clustered
    pub(crate) speed_thres: f64,
    /// Maximum time interval before any succeeding points are left out of cluster
    pub(crate) max_time_thres: TimeDelta,
}

impl<Dist> DbScanConf<Dist>
where
    Dist: Fn(&PointM, &PointM) -> f64,
{
    pub fn run<'p>(&self, points: &'p [PointM], dist_thres: f64) -> Vec<()> {
        let mut cluster_label = 0_u64;
        let mut labels: HashMap<usize, Option<u64>> = HashMap::with_capacity(points.len());
        let mut classes: Vec<Classification> = Vec::with_capacity(points.len());

        for (idx, ele) in points.iter().enumerate() {
            if !labels.contains_key(&idx) {
                //unprocessed
                let mut neighbors = self.range_query(ele, &points, dist_thres); //TODO: potentially expensive

                if (neighbors.len() as u64) < self.min_cluster_size.get() {
                    classes.push(Classification::Unclassified(ele));
                    labels.entry(idx).insert_entry(None);
                }

                cluster_label += 1;
                labels.entry(idx).insert_entry(Some(cluster_label));

                for (nidx, nele) in neighbors {
                    let _ = labels
                        .entry(nidx)
                        .and_modify(|v| *v = Some(v.unwrap_or(cluster_label))).or_insert(Some(cluster_label)); //wikipedia pseudocode is confusing

                    if !labels.contains_key(&nidx) {
                        let n_neighbors = self.range_query(nele, &points, dist_thres);

                        if (n_neighbors.len() as u64) >= self.min_cluster_size.get() {
                            neighbors.push((0,nele))// TODO: finish
                        }
                    }
                }
            }
        }

        todo!()
    }
    //TODO: maybe range query should be performed with the help of an r-tree, but that necesitates a points table
    fn range_query<'p>(
        &self,
        qp: &PointM,
        points: &'p [PointM],
        dist_thres: f64,
    ) -> Vec<(usize, &'p PointM)> {
        let mut neighbors: Vec<(usize, &PointM)> = Vec::with_capacity(10);
        // TODO: takewhile, if point n is not neighbor to qp, then n+1 is neither (after some threshold)
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

    use crate::algo::stop_cluster::DbScanConf;

    #[test]
    fn build_conf() {
        let conf = DbScanConf::builder()
            .dist(|a, b| 0.0)
            .max_time_thres(TimeDelta::zero())
            .speed_thres(1.5)
            .min_cluster_size(42.try_into().unwrap())
            .build();

        assert_eq!(conf.min_cluster_size, 42.try_into().unwrap());
        assert_eq!(conf.max_time_thres, TimeDelta::zero());
        assert_eq!(conf.speed_thres, 1.5);
    }
}
