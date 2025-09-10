pub struct GPSPosition {
    pub mmsi: Vec<u64>,
    pub a: Vec<u16>,
    pub b: Vec<u16>,
    pub c: Vec<u16>,
    pub d: Vec<u16>,
}

impl GPSPosition {
    pub fn new() -> Self {
        Self {
            mmsi: Vec::new(),
            a: Vec::new(),
            b: Vec::new(),
            c: Vec::new(),
            d: Vec::new(),
        }
    }
}

