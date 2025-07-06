use ej_dispatcher_sdk::{
    EjRunResult, ejjob::EjJobApi, fetch_jobs::fetch_jobs, fetch_run_result::fetch_run_result,
};
use tracing::{info, warn};

use crate::{Ctx, prelude::*};

pub async fn fetch_latest_job_result_from_commit(
    ctx: &Ctx,
    commit: String,
) -> Result<Option<EjRunResult>> {
    info!("Fecthing jobs associated with commit {commit}");
    let mut jobs = fetch_jobs(&ctx.socket, commit.clone()).await?;
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

    Ok(Some(fetch_run_result(&ctx.socket, job.id).await?))
}
