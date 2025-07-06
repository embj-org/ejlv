use std::{path::PathBuf, time::Duration};

use crate::cli::{Cli, Commands, DispatchArgs, PrArgs};
use crate::comment::generate_comment;
use crate::ej::fetch_latest_job_result_from_commit;
use crate::gh::{get_latest_master_commit, get_pr_comment};
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
    pub token: Option<String>,
    pub socket: PathBuf,
}

impl Ctx {
    pub fn new(
        socket: PathBuf,
        gh_repo: impl Into<String>,
        gh_owner: impl Into<String>,
        token: Option<String>,
    ) -> Self {
        Self {
            socket,
            gh_repo: gh_repo.into(),
            gh_owner: gh_owner.into(),
            token,
        }
    }
    pub fn build_octocrab(&self) -> Result<Octocrab> {
        if let Some(token) = &self.token {
            Ok(Octocrab::builder()
                .personal_token(token.to_string())
                .build()?)
        } else {
            Ok(Octocrab::builder().build()?)
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

pub async fn on_run(ctx: Ctx, job: DispatchArgs, pr: Option<PrArgs>) -> Result<()> {
    let octocrab = ctx.build_octocrab()?;
    info!("Dispatching run");
    let result = dispatch_run(
        &ctx.socket,
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
        fetch_latest_job_result_from_commit(&ctx, latest_master_commit).await?
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
    debug!(comment_body);

    if let Some(pr) = pr {
        let pr_comment = get_pr_comment(&ctx, &octocrab, pr.pr_number).await?;
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
                .create_comment(pr.pr_number, comment_body)
                .await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::DispatchBuild { socket, job } => on_build(socket, job).await,
        Commands::DispatchRun { socket, job, pr } => {
            let ctx = if let Some(pr) = &pr {
                Ctx::new(socket, "lvgl", "lvgl", Some(pr.gh_token.clone()))
            } else {
                Ctx::new(socket, "lvgl", "lvgl", None)
            };
            on_run(ctx, job, pr).await
        }
    }
}
