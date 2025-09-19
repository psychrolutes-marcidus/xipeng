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

pub fn nav_status_converter(field: &str) -> Option<NavStatusValue> {
    match field {
        "Under way using engine" => Some(NavStatusValue::UnderWayUsingEngine),
        "At anchor" => Some(NavStatusValue::Anchored),
        "Not under command" => Some(NavStatusValue::NotUnderCommand),
        "Restricted maneuverability" => Some(NavStatusValue::RestrictedManeuverability),
        "Constrained by her draught" => Some(NavStatusValue::ConstrainedByHerDraught),
        "Moored" => Some(NavStatusValue::Moord),
        "Aground" => Some(NavStatusValue::Aground),
        "Engaged in fishing" => Some(NavStatusValue::EngagedInFishingActivity),
        "Under way sailing" => Some(NavStatusValue::UnderwaySailing),
        "AIS-SART (active)" => Some(NavStatusValue::AISSART),
        _ => None,
    }
}
