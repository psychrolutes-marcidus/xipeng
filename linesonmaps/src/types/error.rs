
//TODO: remove once all error variants are discovered
#[non_exhaustive]
#[derive(PartialEq, Eq,Clone,Debug)]
pub enum Error{
    /// A linestring must have length 0 or >=2
    NumPoints,
    /// Points must be ordered by time (increasing order), and no two points may have the same timestamp
    Timestamp,
}
