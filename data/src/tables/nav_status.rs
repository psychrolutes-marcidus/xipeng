use super::*;

pub struct NavStatus {
    pub mmsi: Vec<u64>,
    pub time_begin: Vec<NaiveDateTime>,
    pub time_end: Vec<NaiveDateTime>,
    pub nav_status: Vec<NavStatusValue>,
}

impl NavStatus {
    pub fn new() -> Self {
        Self {
            mmsi: Vec::new(),
            time_begin: Vec::new(),
            time_end: Vec::new(),
            nav_status: Vec::new(),
        }
    }
}

impl NavStatus {
    pub fn search_by_key(&self, mmsi: MMSIType, time: TimeType) -> Result<NavStatusValue, TabelError> {
        let index = self.mmsi.iter().zip(self.time_begin.iter().zip(self.time_end.iter())).position(|(m, (tb, te))| *m == mmsi && *tb <= time && *te >= time).ok_or(TabelError::MissingKey)?;

        Ok(self.nav_status[index])
    }
}

#[derive(Debug, Copy, Clone)]
pub enum NavStatusValue {
    UnderWayUsingEngine,
    Anchored,
    NotUnderCommand,
    RestrictedManeuverability,
    ConstrainedByHerDraught,
    Moord,
    Aground,
    EngagedInFishingActivity,
    UnderwaySailing,
    AISSART,
}
