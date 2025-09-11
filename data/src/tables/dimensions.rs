use super::*;

pub type DimensionType = u16;

pub struct Dimensions {
    pub mmsi: Vec<MMSIType>,
    pub width: Vec<DimensionType>,
    pub length: Vec<DimensionType>,
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

impl Dimensions {
    pub fn search_by_key(&self, mmsi: MMSIType) -> Result<(DimensionType, DimensionType), TabelError> {
        let index = self.mmsi.iter().position(|x| *x == mmsi).ok_or(TabelError::MissingKey)?;

        Ok((self.width[index], self.length[index]))
    }
}
