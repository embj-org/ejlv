//! Command-line interface definitions for ejlv_cli.
//!
//! Defines the CLI structure, commands, and arguments for this tool

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// EJ Command Line Interface for testing and system setup.
#[derive(Parser)]
#[command(name = "ejlv_cli")]
#[command(about = "EJ LVGL CLI - Job handler for the LVGL's EJ workspace")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands for the EJ CLI testing and setup tool.
#[derive(Subcommand)]
pub enum Commands {
    /// Dispatch a test build job
    DispatchBuild {
        /// Path to the EJD's unix socket
        #[arg(short, long)]
        socket: PathBuf,
        #[command(flatten)]
        job: DispatchArgs,
    },

    /// Dispatch a test run job
    DispatchRun {
        /// Path to the EJD's unix socket
        #[arg(short, long)]
        socket: PathBuf,

        /// Path to the output comment (.md)
        #[arg(long)]
        comment_path: PathBuf,

        #[command(flatten)]
        job: DispatchArgs,
    },

    /// Comment PR
    CommentPR {
        /// Path to the output comment (.md)
        #[arg(long)]
        comment_path: PathBuf,

        /// PR number associated with this run
        #[arg(long)]
        pr_number: u64,

        /// Github token with write access
        #[arg(long)]
        gh_token: String,

        /// A comment (hidden) signature
        #[arg(long)]
        signature: String,
    },
}

/// Arguments for dispatching a job.
#[derive(Args)]
pub struct DispatchArgs {
    /// The maximum job duration in seconds
    #[arg(long)]
    pub seconds: u64,

    /// Git commit hash
    #[arg(long)]
    pub commit_hash: String,

    /// Git remote url
    #[arg(long)]
    pub remote_url: String,

    /// Optional git remote token
    #[arg(long)]
    pub remote_token: Option<String>,
}
