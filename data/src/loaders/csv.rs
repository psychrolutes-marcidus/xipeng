use serde::Deserialize;

use chrono::prelude::*;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct CsvData {
    #[serde(rename = "# Timestamp")]
    pub timestamp: NaiveDateTime,
    #[serde(rename = "Type of mobile")]
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

#[derive(Debug)]
pub enum CsvError {
    CouldNotOpenFile,
    Deserialize,
}

pub fn read_data(path: &str) -> Result<Vec<CsvData>, CsvError> {
    let file = std::fs::File::open(path).map_err(|_| CsvError::CouldNotOpenFile)?;

    let mut reader = csv::Reader::from_reader(file);

    // let data: Vec<CsvData> = reader.deserialize::<CsvData>().map(|x| dbg!(x).map_err(|_| CsvError::Deserialize)).collect::<Result<Vec<CsvData>, CsvError>>()?;

    let mut data: Vec<CsvData> = Vec::new();

    for record in reader.deserialize() {
        let entry: CsvData = dbg!(record).map_err(|_| CsvError::Deserialize)?;

        data.push(entry);
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;

    #[test]
    fn read_csv_data() {
        let temp_dir = tempdir().expect("Could not create temp dir");

        let csv_data_string = b"# Timestamp,Type of mobile,MMSI,Latitude,Longitude,Navigational status,ROT,SOG,COG,Heading,IMO,Callsign,Name,Ship type,Cargo type,Width,Length,Type of position fixing device,Draught,Destination,ETA,Data source type,A,B,C,D
01/09/2025 00:00:00,Base Station,2190064,56.716570,11.519047,Unknown value,,,,,Unknown,Unknown,,Undefined,,,,GPS,,Unknown,,AIS,,,,
01/09/2025 00:00:00,Class A,219024000,57.717413,10.586715,Engaged in fishing,0.0,0.0,4.8,309,Unknown,Unknown,,Undefined,,,,Undefined,,Unknown,,AIS,,,,";

        let mut filepath = temp_dir.keep();

        filepath.push("data.csv");

        let mut file = std::fs::File::create(&filepath).expect("Could not create temp file");

        file.write_all(csv_data_string).expect("Could not write data");

        drop(file);

        let data = read_data(filepath.to_str().unwrap()).expect("Failed to get data");

        dbg!(data);

    }
}
