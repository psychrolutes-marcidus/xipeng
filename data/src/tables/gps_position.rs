use super::*;

pub struct GPSPosition {
    pub mmsi: Vec<MMSIType>,
    pub a: Vec<dimensions::DimensionType>,
    pub b: Vec<dimensions::DimensionType>,
    pub c: Vec<dimensions::DimensionType>,
    pub d: Vec<dimensions::DimensionType>,
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

impl GPSPosition {
    pub fn search_by_key(&self, mmsi: MMSIType) -> Result<(dimensions::DimensionType, dimensions::DimensionType, dimensions::DimensionType, dimensions::DimensionType), TabelError> {
        let index = self.mmsi.iter().position(|x| *x == mmsi).ok_or(TabelError::MissingKey)?;

        Ok((self.a[index], self.b[index], self.c[index], self.d[index]))
    }
}


