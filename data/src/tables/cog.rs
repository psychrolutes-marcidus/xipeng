use super::*;

type CogType = f32;

pub struct Cog {
    pub mmsi: Vec<MMSIType>,
    pub time: Vec<TimeType>,
    pub cog: Vec<CogType>,
}

impl Cog {
    pub fn new() -> Self {
        Self {
            mmsi: Vec::new(),
            time: Vec::new(),
            cog: Vec::new(),
        }
    }
}

impl Default for Cog {
    fn default() -> Self {
        Self::new()
    }
}

impl Cog {
    pub fn search_by_key(&self, mmsi: MMSIType, time: TimeType) -> Result<CogType, TableError> {
        let index = self
            .mmsi
            .iter()
            .zip(self.time.iter())
            .position(|(m, t)| *m == mmsi && *t == time)
            .ok_or(TableError::MissingKey)?;

        Ok(self.cog[index])
    }
}
