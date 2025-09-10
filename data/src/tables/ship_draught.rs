use super::*;

pub struct Draught {
    pub mmsi: Vec<u64>,
    pub time_begin: Vec<NaiveDateTime>,
    pub time_end: Vec<NaiveDateTime>,
    pub draught: Vec<f32>,
}

impl Draught {
    pub fn new() -> Self {
        Self {
            mmsi: Vec::new(),
            time_begin: Vec::new(),
            time_end: Vec::new(),
            draught: Vec::new() ,
       }
    }
}
