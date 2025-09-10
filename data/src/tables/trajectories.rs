use super::*;
use geo_types::LineString;

pub struct Trajectories {
    pub mmsi: Vec<u64>,
    pub trajectory: Vec<LineString>, // Change this to the custom linestringm type.
}

impl Trajectories {
    pub fn new() -> Self {
        Self {
            mmsi: Vec::new(),
            trajectory: Vec::new(),
        }
    }
}

