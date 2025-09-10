use serde::Deserialize;

use chrono::prelude::*;

pub mod tables;

#[derive(Debug, Deserialize)]
pub struct CsvData {
    pub timestamp: NaiveDateTime,
    pub type_of_mobile: String,
    pub mmsi: u64,
    pub latitude: f64,
    pub longitude: f64,
    pub nav_status: String,
    pub rot: Option<f64>,
    pub sog: Option<f64>,
    pub cog: Option<f64>,
    pub heading: Option<u16>,
    pub imo: String,
    pub callsign: String,
    pub name: Option<String>,
    pub ship_type: String,
    pub cargo_type: Option<String>,
    pub width: Option<u16>,
    pub length: Option<u16>,
    pub position_fixing_device: String,
    pub draught: Option<f64>,
    pub destination: String,
    pub eta: Option<NaiveDateTime>,
    pub data_source_type: String,
    pub a: Option<u16>,
    pub b: Option<u16>,
    pub c: Option<u16>,
    pub d: Option<u16>,
}


