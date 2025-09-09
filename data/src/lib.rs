use chrono::prelude::*;

struct CsvData {
    timestamp: NaiveDateTime,
    type_of_mobile: String,
    MMSI: u64,
    latitude: f32,
    longitude: f32,
    nav_status: String,

}
