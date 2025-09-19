use super::*;

use linesonmaps::types::linestringm::LineStringM;

pub struct Trajectories {
    pub mmsi: Vec<MMSIType>,
    pub trajectory: Vec<LineStringM<4326>>, // Change this to the custom linestringm type.
}

impl Trajectories {
    pub fn new() -> Self {
        Self {
            mmsi: Vec::new(),
            trajectory: Vec::new(),
        }
    }
}

impl Default for Trajectories {
    fn default() -> Self {
        Self::new()
    }
}

impl Trajectories {
    pub fn search_by_key(&self, mmsi: MMSIType) -> Result<&LineStringM, TableError> {
        let index = self
            .mmsi
            .iter()
            .position(|m| *m == mmsi)
            .ok_or(TableError::MissingKey)?;

        Ok(&self.trajectory[index])
    }
}

