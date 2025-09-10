use super::*;

use geo_types::Geometry;

pub struct StopObject {
    pub mmsi: Vec<u64>,
    pub time_begin: Vec<NaiveDateTime>,
    pub time_end: Vec<NaiveDateTime>,
    pub geom: Vec<Geometry>,
}

impl StopObject {
    pub fn new() -> Self {
        Self {
            mmsi: Vec::new(),
            time_begin: Vec::new(),
            time_end: Vec::new(),
            geom: Vec::new(),
       }
    }
}
