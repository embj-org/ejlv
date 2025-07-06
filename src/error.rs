use std::num::ParseIntError;

/// Main error type
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DispactherSDK(#[from] ej_dispatcher_sdk::error::Error),

    #[error("Invalid column at line {0}, col {1}: '{2}'")]
    InvalidResultColumn(usize, usize, String),

    #[error("Parse failed for column at line {0}, col {1}: '{2}' - {3}")]
    ParseIntFailed(usize, usize, String, ParseIntError),

    #[error("Failed to find scene '{0}'")]
    SceneMissing(String),

    #[error(transparent)]
    Octocrab(#[from] octocrab::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Failed to fetch latest master commit")]
    FailedToFetchMasterCommit,
}
