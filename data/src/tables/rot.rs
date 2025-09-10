use super::*;

pub struct Rot {
    pub mmsi: Vec<u64>,
    pub time: Vec<NaiveDateTime>,
    pub rot: Vec<f32>,
}

impl Rot {
    pub fn new() -> Self {
        Self { mmsi: Vec::new(), time: Vec::new(), rot: Vec::new() }
    }
}
