
//TODO: remove once all error variants are discovered
#[non_exhaustive]
pub enum Error{
    /// A linestring must have length 0 or >=2
    InvalidLinestring,
}
