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
