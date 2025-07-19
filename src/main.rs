use std::{path::PathBuf, time::Duration};

use crate::chart::{COLORS, RunResult, create_comparison_chart};
use crate::cli::{Cli, Commands, DispatchArgs};
use crate::comment::generate_comment;
use crate::ej::fetch_latest_run_result_from_commit;
use crate::gh::{add_comment_signature, get_latest_master_commit, get_pr_comment};
use crate::parser::{parse_run_result, parse_scenes};
use crate::prelude::*;
use crate::result::calculate_result_delta;
use crate::scene::SceneMetric;
use clap::Parser;
use ej_dispatcher_sdk::{dispatch_build, dispatch_run};
mod chart;
mod cli;
mod comment;
mod ej;
mod error;
mod gh;
mod parser;
mod prelude;
mod result;
mod scene;
use octocrab::Octocrab;
use plotters::prelude::{IntoDrawingArea, SVGBackend};
use plotters::style::RGBColor;
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

pub fn create_benchmark_graph(
    input_dir: PathBuf,
    output: PathBuf,
    metric: SceneMetric,
    h_res: u32,
    v_res: u32,
) -> Result<()> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(input_dir)?
        .into_iter()
        .map(|dir_entry| dir_entry.expect("Invalid dir_entry").path())
        .collect();

    // So multiple runs with the same input produce the same graph
    paths.sort();

    let mut run_results = Vec::new();
    for path in paths {
        if !path.is_file() {
            continue;
        }
        let raw_results = std::fs::read_to_string(&path)?;
        let scenes = parse_scenes(&raw_results)?;
        let run_name = path
            .file_stem()
            .ok_or(Error::FailedToGetFileName(path.clone()))?
            .to_str()
            .ok_or(Error::FilePathConversionFailed(path.clone()))?;

        let result = RunResult::new(run_name, scenes);
        run_results.push(result);
    }

    let root = SVGBackend::new(&output, (1200, 800)).into_drawing_area();
    root.fill(&RGBColor(245, 245, 245))?;

    let mut colors = COLORS.to_vec();

    // TODO: Generate some other colors here to avoid duplication
    let mut i = 0;
    while run_results.len() > colors.len() {
        colors.push(colors[i]);
        i = (i + 1) % colors.len();
    }
    let title = format!("{}x{}", h_res, v_res);
    create_comparison_chart(&root, &title, run_results.as_slice(), &metric, &colors)?;
    root.present()?;
    Ok(())
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
        fetch_latest_run_result_from_commit(&socket, latest_master_commit).await?
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
        Commands::BenchmarkGraph {
            input_dir,
            output,
            metric,
            h_res,
            v_res,
        } => create_benchmark_graph(input_dir, output, metric, h_res, v_res),
    }
}
