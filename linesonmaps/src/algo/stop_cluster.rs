use std::num::NonZero;

use crate::types::pointm::PointM;

pub enum Classification {
    Core(PointM),
    Edge(PointM),
    Unclassified(PointM),
}

pub struct DbScanConf<Dist>
where
    Dist: Fn(PointM, PointM) -> f64,
{
    min_cluster_size: Option<NonZero<u64>>,
    dist_thres: Option<Dist>,
    
}
