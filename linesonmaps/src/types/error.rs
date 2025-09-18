use thiserror::Error;

//TODO: remove once all error variants are discovered
#[non_exhaustive]
#[derive(PartialEq, Eq, Clone, Debug,Error)]
pub enum Error {
    #[error("Illegal Linestring with length 1")]
    NumPoints,
    #[error("Linestring points must temporally ordered")]
    Timestamp,
    #[error("tried to convert to wrong geometry sub-type")]
    IncompatibleType,
    #[error("Geometry unexpectedly empty")]
    Empty,
    #[error("tried to read from a non-existent dimension")]
    Dimension,
}
