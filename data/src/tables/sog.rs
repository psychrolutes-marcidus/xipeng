use super::*;

pub type SogType = f32;

pub struct Sog {
    pub mmsi: Vec<MMSIType>,
    pub time: Vec<TimeType>,
    pub sog: Vec<SogType>,
}

impl Sog {
    pub fn new() -> Self {
        Self {
            mmsi: Vec::new(),
            time: Vec::new(),
            sog: Vec::new(),
        }
    }
}

impl Default for Sog {
    fn default() -> Self {
        Self::new()
    }
}

impl Sog {
    pub fn search_by_key(&self, mmsi: MMSIType, time: TimeType) -> Result<SogType, TabelError> {
        let index = self
            .mmsi
            .iter()
            .zip(self.time.iter())
            .position(|(m, t)| *m == mmsi && *t == time)
            .ok_or(TabelError::MissingKey)?;

        Ok(self.sog[index])
    }
}
