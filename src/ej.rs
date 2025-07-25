use std::path::Path;

use ej_dispatcher_sdk::{
    EjJobType, EjRunResult, ejjob::EjJobApi, fetch_jobs::fetch_jobs,
    fetch_run_result::fetch_run_result,
};
use tracing::{info, warn};

use crate::prelude::*;

pub async fn fetch_latest_run_result_from_commit(
    socket: &Path,
    commit: String,
) -> Result<Option<EjRunResult>> {
    info!("Fecthing jobs associated with commit {commit}");
    let mut jobs = fetch_jobs(socket, commit.clone()).await?;
    jobs = jobs
        .into_iter()
        .filter(|job| job.job_type == EjJobType::BuildAndRun)
        .collect();
    if jobs.len() > 1 {
        warn!("Found multiple jobs associated with commit '{commit}'. Using latest one");
        EjJobApi::sort_by_finished_desc(&mut jobs);
    } else if jobs.len() == 1 {
        info!("Found 1 job associate with commit '{commit}'");
    } else {
        info!("No job associated with commit '{commit}'");
        return Ok(None);
    };
    let job = &jobs[0];

    Ok(Some(fetch_run_result(socket, job.id).await?))
}
