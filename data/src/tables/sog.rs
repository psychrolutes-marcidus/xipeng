use super::*;

pub type SogType = f32;

pub struct Sog {
    pub mmsi: Vec<MMSIType>,
    pub time: Vec<TimeType>,
    pub sog: Vec<SogType>,
    pub b_tree_index: std::collections::BTreeMap<(MMSIType, TimeType), usize>,
}

impl Sog {
    pub fn new() -> Self {
        Self {
            mmsi: Vec::new(),
            time: Vec::new(),
            sog: Vec::new(),
            b_tree_index: std::collections::BTreeMap::new(),
        }
    }
}

impl Default for Sog {
    fn default() -> Self {
        Self::new()
    }
}

impl Sog {
    pub fn search_by_key(&self, mmsi: MMSIType, time: TimeType) -> Result<SogType, TableError> {
        let index = self
            .mmsi
            .iter()
            .zip(self.time.iter())
            .position(|(m, t)| *m == mmsi && *t == time)
            .ok_or(TableError::MissingKey)?;

        Ok(self.sog[index])
    }
}
