use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error in query: {}", _0)]
    InvalidQuery(String)
}
