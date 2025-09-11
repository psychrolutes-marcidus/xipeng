use super::*;

use linesonmaps::types::linestringm::LineStringM;

pub struct Trajectories {
    pub mmsi: Vec<MMSIType>,
    pub trajectory: Vec<LineStringM>, // Change this to the custom linestringm type.
}

impl Trajectories {
    pub fn new() -> Self {
        Self {
            mmsi: Vec::new(),
            trajectory: Vec::new(),
        }
    }
}

impl Trajectories {
    pub fn search_by_key(&self, mmsi: MMSIType) -> Result<&LineStringM, TabelError> {
        let index = self.mmsi.iter().position(|m| *m == mmsi).ok_or(TabelError::MissingKey)?;

        Ok(&self.trajectory[index])
    }
}

