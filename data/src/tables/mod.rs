pub use chrono::prelude::*;
pub use std::collections;
use std::error::Error;
use std::fmt::Display;

pub mod nav_status;
pub mod ship_draught;
pub mod cog;
pub mod sog;
pub mod rot;
pub mod gps_position;
pub mod dimensions;
pub mod stop_object;
pub mod trajectories;

type TimeType = NaiveDateTime;
type MMSIType = u64;

pub struct Ships {
    pub nav_status: nav_status::NavStatus,
    pub ship_draught: ship_draught::Draught,
    pub cog: cog::Cog,
    pub sog: sog::Sog,
    pub rot: rot::Rot,
    pub gps_position: gps_position::GPSPosition,
    pub dimensions: dimensions::Dimensions,
}

#[derive(Debug)]
pub enum TabelError {
    MissingKey,
    DuplicateKey
}

impl Display for TabelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingKey => write!(f, "Could not find provided key in table"),
            Self::DuplicateKey => write!(f, "Key already exists in table"),
        }
    }
}

impl Error for TabelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
