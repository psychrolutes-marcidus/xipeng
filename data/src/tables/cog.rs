use super::*;


type CogType = f32

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

impl Cog {
    pub fn search_by_key(&self, mmsi: MMSIType, time: TimeType) -> Result<CogType, TabelError> {
        let index = self.mmsi.into_iter().zip(self.time.into_iter()).enumerate().filter(|(_, (m,t))| *m == mmsi && *t == time).map(|(i, _)| i).last().ok_or(TabelError::MissingKey)?;

        Ok(self.cog[index])
    }
}
