use super::*;

use geo_types::Geometry;

pub struct StopObject {
    pub mmsi: Vec<MMSIType>,
    pub time_begin: Vec<TimeType>,
    pub time_end: Vec<TimeType>,
    pub geom: Vec<Geometry>, // It's a placeholder type
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

impl Default for StopObject {
    fn default() -> Self {
        Self::new()
    }
}

impl StopObject {
    pub fn search_by_key(&self, mmsi: MMSIType, time: TimeType) -> Result<&Geometry, TableError> {
        let index = self
            .mmsi
            .iter()
            .zip(self.time_begin.iter().zip(self.time_end.iter()))
            .position(|(m, (tb, te))| *m == mmsi && *tb <= time && *te >= time)
            .ok_or(TableError::MissingKey)?;

        Ok(&self.geom[index])
    }
}
