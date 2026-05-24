pub type Result<T> = std::result::Result<T, error::Error>;

mod blockhash;
pub mod constants;
pub mod error;
pub mod hasher;
mod roll;
