use super::*;

pub type DraughtType = f32;

pub struct Draught {
    pub mmsi: Vec<MMSIType>,
    pub time_begin: Vec<TimeType>,
    pub time_end: Vec<TimeType>,
    pub draught: Vec<DraughtType>,
}

impl Draught {
    pub fn new() -> Self {
        Self {
            mmsi: Vec::new(),
            time_begin: Vec::new(),
            time_end: Vec::new(),
            draught: Vec::new(),
        }
    }
}

impl Default for Draught {
    fn default() -> Self {
        Self::new()
    }
}

impl Draught {
    pub fn search_by_key(&self, mmsi: MMSIType, time: TimeType) -> Result<DraughtType, TableError> {
        let index = self
            .mmsi
            .iter()
            .zip(self.time_begin.iter().zip(self.time_end.iter()))
            .position(|(m, (tb, te))| *m == mmsi && *tb <= time && *te >= time)
            .ok_or(TableError::MissingKey)?;

        Ok(self.draught[index])
    }

    pub fn search_range_by_time(&self, mmsi: MMSIType, time_from: TimeType, time_to: TimeType) -> Vec<usize> {
        self.mmsi.iter().zip(self.time_begin.iter().zip(self.time_end.iter())).enumerate().filter(|(_,(m,(tb,te)))| **m == mmsi && time_from <= **te && time_to >= **tb).map(|(i,_)| i).collect()
    }
}
