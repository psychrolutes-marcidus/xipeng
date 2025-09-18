use serde::Deserialize;

use chrono::prelude::*;
use crate::errors::*;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct CsvData {
    #[serde(rename = "# Timestamp")]
    pub timestamp: String,
    #[serde(rename = "Type of mobile")]
    pub type_of_mobile: String,
    #[serde(rename = "MMSI")]
    pub mmsi: u64,
    #[serde(rename = "Latitude")]
    pub latitude: f64,
    #[serde(rename = "Longitude")]
    pub longitude: f64,
    #[serde(rename = "Navigational status")]
    pub nav_status: String,
    #[serde(rename = "ROT")]
    pub rot: Option<f64>,
    #[serde(rename = "SOG")]
    pub sog: Option<f64>,
    #[serde(rename = "COG")]
    pub cog: Option<f64>,
    #[serde(rename = "Heading")]
    pub heading: Option<u16>,
    #[serde(rename = "IMO")]
    pub imo: String,
    #[serde(rename = "Callsign")]
    pub callsign: String,
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(rename = "Ship type")]
    pub ship_type: String,
    #[serde(rename = "Cargo type")]
    pub cargo_type: Option<String>,
    #[serde(rename = "Width")]
    pub width: Option<u16>,
    #[serde(rename = "Length")]
    pub length: Option<u16>,
    #[serde(rename = "Type of position fixing device")]
    pub position_fixing_device: String,
    #[serde(rename = "Draught")]
    pub draught: Option<f64>,
    #[serde(rename = "Destination")]
    pub destination: String,
    #[serde(rename = "ETA")]
    pub eta: Option<NaiveDateTime>,
    #[serde(rename = "Data source type")]
    pub data_source_type: String,
    #[serde(rename = "A")]
    pub a: Option<u16>,
    #[serde(rename = "B")]
    pub b: Option<u16>,
    #[serde(rename = "C")]
    pub c: Option<u16>,
    #[serde(rename = "D")]
    pub d: Option<u16>,
}

pub fn read_data(path: &str) -> Result<Vec<CsvData>, CsvError> {
    let file = std::fs::File::open(path).map_err(|_| CsvError::CouldNotOpenFile)?;

    let mut reader = csv::Reader::from_reader(file);

    let data: Vec<CsvData> = reader
        .deserialize::<CsvData>()
        .map(|x| dbg!(x).map_err(|_| CsvError::Deserialize))
        .collect::<Result<Vec<CsvData>, CsvError>>()?;

    Ok(data)
}

pub fn time_converter(time: &str) -> Result<NaiveDateTime, CsvError> {
    NaiveDateTime::parse_from_str(time, "%d/%m/%Y %H:%M:%S").map_err(|_| CsvError::TimeConvertError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn read_csv_data() {
        let temp_dir = tempdir().expect("Could not create temp dir");

        let csv_data_string = b"# Timestamp,Type of mobile,MMSI,Latitude,Longitude,Navigational status,ROT,SOG,COG,Heading,IMO,Callsign,Name,Ship type,Cargo type,Width,Length,Type of position fixing device,Draught,Destination,ETA,Data source type,A,B,C,D
01/09/2025 00:00:00,Base Station,2190064,56.716570,11.519047,Unknown value,,,,,Unknown,Unknown,,Undefined,,,,GPS,,Unknown,,AIS,,,,
01/09/2025 00:00:00,Class A,219024000,57.717413,10.586715,Engaged in fishing,0.0,0.0,4.8,309,Unknown,Unknown,,Undefined,,,,Undefined,,Unknown,,AIS,,,,";

        let mut filepath = temp_dir.keep();

        filepath.push("data.csv");

        let mut file = std::fs::File::create(&filepath).expect("Could not create temp file");

        file.write_all(csv_data_string)
            .expect("Could not write data");

        let data = read_data(filepath.to_str().unwrap()).expect("Failed to get data");

        assert_eq!(data[1].mmsi, 219024000);
    }
}
