pub struct Dimensions {
    pub mmsi: Vec<u64>,
    pub width: Vec<u16>,
    pub length: Vec<u16>,
}

impl Dimensions {
    pub fn new() -> Self {
        Self {
            mmsi: Vec::new(),
            width: Vec::new(),
            length: Vec::new(),
        }
    }
}
