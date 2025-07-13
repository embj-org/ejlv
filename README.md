# ejlv

A command-line interface tool for dispatching jobs to EJD (EJ Dispatcher) in the [LVGL](https://github.com/lvgl/lvgl.git) testing workspace.

## Overview

`ejlv` is a CLI tool designed to interface with the EJ framework's dispatcher component (EJD).
This tool is used by [LVGL](https://lvgl.io)'s CI pipeline [Github](https://github.com/lvgl/lvgl) to dispatch jobs, collect results and comment the pull request with the findings.

## Features

- **Job Dispatching**: Submit build and run jobs to EJD
- **Real-time Results**: Collect and display job results directly in the terminal
- **CI/CD Integration**: Designed for seamless integration with GitHub Actions
- **Socket Communication**: Uses Unix socket interface for efficient communication with EJD
- **Pull Request Support**: Built-in support for PR-based testing workflows

## Installation

```bash
cargo install ejlv
```

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
