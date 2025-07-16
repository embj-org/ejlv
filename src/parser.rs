use std::num::ParseIntError;

use ej_config::ej_board_config::EjBoardConfigApi;
use ej_dispatcher_sdk::EjRunResult;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Scene {
    pub scene_name: String,
    pub avg_cpu: i32,
    pub avg_fps: i32,
    pub avg_time: i32,
    pub render_time: i32,
    pub flush_time: i32,
}
pub fn parse_run_result(result: EjRunResult) -> Result<Vec<(EjBoardConfigApi, Vec<Scene>)>> {
    let mut results = Vec::new();
    for (board_config, result) in result.results {
        results.push((board_config, parse_result(&result)?));
    }
    Ok(results)
}

pub fn parse_result(result: &str) -> Result<Vec<Scene>> {
    let mut found_start_of_results = false;
    let mut found_header = false;
    let mut scenes = Vec::new();

    for mut line in result.lines() {
        if line.starts_with("Benchmark Summary") {
            found_start_of_results = true;
            continue;
        }
        if !found_start_of_results {
            continue;
        }
        line = line.trim();
        let mut cols: Vec<&str> = line.split(",").collect();
        if cols.len() < 6 {
            break;
        }
        if !found_header {
            found_header = true;
            continue;
        }
        let line_num = scenes.len();
        cols[1] = cols[1].strip_suffix('%').ok_or(Error::InvalidResultColumn(
            line_num,
            1,
            cols[1].to_string(),
        ))?;

        let avg_cpu: i32 = parse_int_col(&cols, line_num, 1)?;
        let avg_fps: i32 = parse_int_col(&cols, line_num, 2)?;
        let avg_time: i32 = parse_int_col(&cols, line_num, 3)?;
        let render_time: i32 = parse_int_col(&cols, line_num, 4)?;
        let flush_time: i32 = parse_int_col(&cols, line_num, 5)?;

        let scene = Scene {
            scene_name: cols[0].to_string(),
            avg_cpu,
            avg_fps,
            avg_time,
            render_time,
            flush_time,
        };
        scenes.push(scene);
    }

    return Ok(scenes);
}

fn parse_int_col(cols: &Vec<&str>, line_num: usize, col_num: usize) -> Result<i32> {
    cols[col_num].trim().parse().map_err(|err: ParseIntError| {
        Error::ParseIntFailed(line_num, col_num, cols[col_num].to_string(), err)
    })
}
