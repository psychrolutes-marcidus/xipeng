use super::*;

pub struct Sog {
    pub mmsi: Vec<u64>,
    pub time: Vec<NaiveDateTime>,
    pub sog: Vec<f32>,
}

impl Sog {
    pub fn new() -> Self {
        Self {
            mmsi: Vec::new(),
            time: Vec::new(),
            sog: Vec::new(),
       }
    }
}
