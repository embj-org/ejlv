use std::{path::PathBuf, time::Duration};

use crate::cli::{Cli, Commands, DispatchArgs};
use crate::comment::generate_comment;
use crate::ej::fetch_latest_job_result_from_commit;
use crate::gh::{add_comment_signature, get_latest_master_commit, get_pr_comment};
use crate::parser::parse_run_result;
use crate::prelude::*;
use crate::result::calculate_result_delta;
use clap::Parser;
use ej_dispatcher_sdk::{dispatch_build, dispatch_run};
mod cli;
mod comment;
mod ej;
mod error;
mod gh;
mod parser;
mod prelude;
mod result;
use octocrab::Octocrab;
use tracing::{debug, error, info};

pub struct Ctx {
    pub gh_repo: String,
    pub gh_owner: String,
}

impl Default for Ctx {
    fn default() -> Self {
        Self::new("lvgl", "lvgl")
    }
}
impl Ctx {
    pub fn new(gh_repo: impl Into<String>, gh_owner: impl Into<String>) -> Self {
        Self {
            gh_repo: gh_repo.into(),
            gh_owner: gh_owner.into(),
        }
    }
}

pub async fn on_build(socket: PathBuf, job: DispatchArgs) -> Result<()> {
    let result = dispatch_build(
        &socket,
        job.commit_hash,
        job.remote_url,
        job.remote_token,
        Duration::from_secs(job.seconds),
    )
    .await?;

    info!("{}", result);
    if result.success {
        Ok(())
    } else {
        Err(Error::DispactherSDK(
            ej_dispatcher_sdk::error::Error::BuildError,
        ))
    }
}

pub async fn on_run(
    ctx: Ctx,
    socket: PathBuf,
    job: DispatchArgs,
    comment_path: PathBuf,
) -> Result<()> {
    let octocrab = Octocrab::builder().build()?;
    info!("Dispatching run");
    let result = dispatch_run(
        &socket,
        job.commit_hash,
        job.remote_url,
        job.remote_token,
        Duration::from_secs(job.seconds),
    )
    .await?;

    if result.success {
        info!("Run Ok");
    } else {
        error!("Run Failed");
    }
    debug!("Job result {}", result);
    let latest_master_commit = get_latest_master_commit(&ctx, &octocrab).await?;
    let master_result = if let Some(result) =
        fetch_latest_job_result_from_commit(&socket, latest_master_commit).await?
    {
        info!("Parsing latest master result");
        parse_run_result(result)?
    } else {
        Vec::new()
    };

    info!("Parsing latest run result");
    let result = parse_run_result(result)?;

    info!("Calculating result difference");
    let result = calculate_result_delta(result, &master_result);

    info!("Generating comment");
    let comment_body = generate_comment(&result);
    tokio::fs::write(&comment_path, comment_body).await?;
    info!("Comment available in {}", comment_path.display());

    Ok(())
}

pub async fn on_comment_pr(
    ctx: Ctx,
    comment_path: PathBuf,
    pr_number: u64,
    gh_token: String,
    signature: String,
) -> Result<()> {
    let octocrab = Octocrab::builder().personal_token(gh_token).build()?;
    let pr_comment = get_pr_comment(&ctx, &octocrab, pr_number, &signature).await?;

    let comment_body = tokio::fs::read_to_string(&comment_path).await?;
    let comment_body = add_comment_signature(comment_body, &signature);

    if let Some(comment) = pr_comment {
        info!("Updating existing comment {}", comment.id);
        octocrab
            .issues(ctx.gh_owner, ctx.gh_repo)
            .update_comment(comment.id, comment_body)
            .await?;
    } else {
        info!("Creating new comment");
        octocrab
            .issues(ctx.gh_owner, ctx.gh_repo)
            .create_comment(pr_number, comment_body)
            .await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::DispatchBuild { socket, job } => on_build(socket, job).await,
        Commands::DispatchRun {
            socket,
            job,
            comment_path,
        } => {
            let ctx = Ctx::default();
            on_run(ctx, socket, job, comment_path).await
        }
        Commands::CommentPR {
            comment_path,
            pr_number,
            gh_token,
            signature,
        } => {
            let ctx = Ctx::default();
            on_comment_pr(ctx, comment_path, pr_number, gh_token, signature).await
        }
    }
}
