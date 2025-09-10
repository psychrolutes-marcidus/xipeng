pub use chrono::prelude::*;
pub use std::collections;

type TimeType = NaiveDateTime;
type MMSIType = u64;

pub enum TabelError {
    MissingKey,
}

pub mod nav_status;
pub mod ship_draught;
pub mod cog;
pub mod sog;
pub mod rot;
pub mod gps_position;
pub mod dimensions;
pub mod stop_object;
pub mod trajectories;

pub struct Ships {
    pub nav_status: nav_status::NavStatus,
    pub ship_draught: ship_draught::Draught,
    pub cog: cog::Cog,
    pub sog: sog::Sog,
    pub rot: rot::Rot,
    pub gps_position: gps_position::GPSPosition,
    pub dimensions: dimensions::Dimensions,
}
