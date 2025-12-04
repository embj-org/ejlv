use std::{num::ParseIntError, path::PathBuf};

use ej_dispatcher_sdk::EjRunResult;
use plotters::prelude::{DrawingBackend, SVGBackend};

/// Main error type
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DispactherSDK(#[from] ej_dispatcher_sdk::error::Error),

    #[error("Invalid column at line {0}, col {1}: '{2}'")]
    InvalidResultColumn(usize, usize, String),

    #[error("Parse failed for column at line {0}, col {1}: '{2}' - {3}")]
    ParseIntFailed(usize, usize, String, ParseIntError),

    #[error("Result slice is empty")]
    ResultSliceEmpty,

    #[error("Failed to find scene '{0}'")]
    SceneMissing(String),

    #[error(transparent)]
    Octocrab(#[from] octocrab::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Failed to fetch latest master commit")]
    FailedToFetchMasterCommit,

    #[error(transparent)]
    Plotters(
        #[from]
        plotters::prelude::DrawingAreaErrorKind<
            <SVGBackend<'static> as DrawingBackend>::ErrorType,
        >,
    ),

    #[error("Invalid Metric {0}")]
    InvalidMetric(String),

    #[error("Failed to get filename from {0}")]
    FailedToGetFileName(PathBuf),

    #[error("Failed to convert file path to string {0}")]
    FilePathConversionFailed(PathBuf),

    #[error("Run error {0:?}")]
    RunError(EjRunResult),
}
