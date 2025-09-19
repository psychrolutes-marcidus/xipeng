use crate::errors::*;
pub use chrono::prelude::*;
use itertools::Itertools;
use std::collections;

use std::fmt::Display;

pub mod cog;
pub mod dimensions;
pub mod gps_position;
pub mod nav_status;
pub mod rot;
pub mod ship_draught;
pub mod sog;
pub mod stop_object;
pub mod trajectories;

type TimeType = DateTime<Utc>;
type MMSIType = i32;

pub struct Ships {
    pub nav_status: nav_status::NavStatus,
    pub ship_draught: ship_draught::Draught,
    pub cog: cog::Cog,
    pub sog: sog::Sog,
    pub rot: rot::Rot,
    pub gps_position: gps_position::GPSPosition,
    pub dimensions: dimensions::Dimensions,
}

