use super::*;

pub struct NavStatus {
    pub mmsi: Vec<MMSIType>,
    pub time_begin: Vec<TimeType>,
    pub time_end: Vec<TimeType>,
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

impl Default for NavStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl NavStatus {
    pub fn search_by_key(
        &self,
        mmsi: MMSIType,
        time: TimeType,
    ) -> Result<NavStatusValue, TabelError> {
        let index = self
            .mmsi
            .iter()
            .zip(self.time_begin.iter().zip(self.time_end.iter()))
            .position(|(m, (tb, te))| *m == mmsi && *tb <= time && *te >= time)
            .ok_or(TabelError::MissingKey)?;

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

pub fn nav_status_converter(field: &str) -> NavStatusValue {
    match field {
        "aground" => NavStatusValue::Aground,
        "ais-sart (active)" => NavStatusValue::AISSART,
        "at anchor" => NavStatusValue::Anchored,
        "constrained by her draught" => NavStatusValue::ConstrainedByHerDraught,
        "engaged in fishing" => NavStatusValue::EngagedInFishingActivity,
        "moored" => NavStatusValue::Moord,
        "not under command" => NavStatusValue::NotUnderCommand,
        "restricted maneuverability" => NavStatusValue::RestrictedManeuverability,
        "under way sailing" => NavStatusValue::UnderwaySailing,
        "under way using engine" => NavStatusValue::UnderWayUsingEngine,
        _ => unreachable!(),
    }
}
