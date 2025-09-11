use super::*;

pub type RotType = f32;

pub struct Rot {
    pub mmsi: Vec<MMSIType>,
    pub time: Vec<TimeType>,
    pub rot: Vec<RotType>,
}

impl Rot {
    pub fn new() -> Self {
        Self { mmsi: Vec::new(), time: Vec::new(), rot: Vec::new() }
    }
}

impl Rot {
    pub fn search_by_key(&self, mmsi: MMSIType, time: TimeType) -> Result<RotType, TabelError> {
        let index = self
            .mmsi
            .iter()
            .zip(self.time.iter())
            .position(|(m, t)| *m == mmsi && *t == time)
            .ok_or(TabelError::MissingKey)?;

        Ok(self.rot[index])
    }
}
