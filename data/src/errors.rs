use postgres::Error as PgError;
use std::num::ParseIntError;
use thiserror::Error;
use std::net::AddrParseError;
use std::env::VarError;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Database Error")]
    Database(#[from] DatabaseError),
    #[error("Table Error")]
    Table(#[from] TabelError),
}

// #[derive(Error, Debug)]
// pub enum CsvError {
//     CouldNotOpenFile,
//     Deserialize,
//     TimeConvertError,
// }

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error")]
    Connect(PgError),
    #[error("Database query error")]
    QueryError(PgError),
    #[error("Invalid Port")]
    PortParse(#[from] ParseIntError),
    #[error("Invalid IP Address")]
    IpAddrParse(#[from] AddrParseError),
    #[error("Missing Environment Variables")]
    MissingEnv(#[from] VarError),


}

#[derive(Error, Debug)]
pub enum TabelError {
    #[error("Could not find key in table")]
    MissingKey,
    #[error("Key already exists in table")]
    DuplicateKey,
    #[error("Error loading data into table")]
    LoaderError,
}

