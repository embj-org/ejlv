# ejlv_cli

A command-line interface tool for dispatching jobs to EJD (EJ Dispatcher) in the [LVGL](https://github.com/lvgl/lvgl.git) testing workspace.

## Overview

ejlv_cli is a CLI tool designed to interface with the EJ framework's dispatcher component (EJD).
This tool is mainly used by [LVGL](https://lvgl.io)'s CI pipeline to dispatch jobs, collect results and comment the pull request with the findings.

## Features

- **Job Dispatching**: Submit build and run jobs to EJD
- **Real-time Results**: Collect and display job results directly in the terminal
- **CI/CD Integration**: Designed for seamless integration with GitHub Actions
- **Socket Communication**: Uses Unix socket interface for efficient communication with EJD
- **Pull Request Support**: Built-in support for PR-based testing workflows

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd ejlv_cli

# Build the CLI tool
cargo build --release

# The binary will be available at target/release/ejlv_cli
```

## Usage

### Basic Commands

ejlv_cli provides two main commands for job dispatching:

#### Dispatch Build Job

Submit a build job to EJD for compilation and building:

```bash
ejlv_cli dispatch-build \
  --socket /path/to/ejd.sock \
  --seconds 300 \
  --commit-hash abc123def456 \
  --remote-url https://github.com/lvgl/lvgl.git \
```

#### Dispatch Run Job

Submit a run job to EJD for testing execution:

```bash
ejlv_cli dispatch-run \
  --socket /path/to/ejd.sock \
  --seconds 600 \
  --commit-hash abc123def456 \
  --remote-url https://github.com/lvgl/lvgl.git \
  --pr-number 123 \
  --gh-token ghp_write_token456
```

### Arguments

#### Common Arguments

- `--socket`: Path to EJD's Unix socket file
- `--seconds`: Maximum job duration in seconds (timeout)
- `--commit-hash`: Git commit hash to test
- `--remote-url`: Git repository URL
- `--remote-token`: Optional Git remote access token

#### Pull Request Arguments (Run Jobs Only)

- `--pr-number`: Pull request number used to comment the PR with the results
- `--gh-token`: GitHub token with write access for updating PR status

## Architecture Integration

ejlv_cli fits into the EJ framework architecture as follows:

```
┌─────────────────┐    ┌─────────────────┐
│   CI/CD         │    │   Developer     │
│                 │    │                 │
└─────────┬───────┘    └─────────┬───────┘
          │                      │
          │                      │
          └──────────────────────┘
                    │
              ┌─────▼─────┐
              │ ejlv_cli  │
              │           │
              └─────┬─────┘
                    │ (Unix Socket)
    ┌───────────────▼────────────┐
    │   EJD (Dispatcher)         │
    │                            │
    │  - Job Queuing             │
    │  - Result Storage          │
    │  - Authentication          │
    └───────────┬────────────────┘
                │
        ┌───────┼───────┐
        │       │       │
   ┌────▼──┐┌───▼──┐┌───▼──┐
   │ EJB 1 ││ EJB 2││ EJB N│
   │       ││      ││      │
   └───────┘└──────┘└──────┘
```
