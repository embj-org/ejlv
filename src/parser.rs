use std::num::ParseIntError;

use ej_config::ej_board_config::EjBoardConfigApi;
use ej_dispatcher_sdk::EjRunResult;

use crate::{prelude::*, scene::Scene};

pub fn parse_run_result(result: EjRunResult) -> Result<Vec<(EjBoardConfigApi, Vec<Scene>)>> {
    let mut results = Vec::new();
    for (board_config, result) in result.results {
        results.push((board_config, parse_scenes(&result)?));
    }
    Ok(results)
}

pub fn parse_scenes(result: &str) -> Result<Vec<Scene>> {
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

#[cfg(test)]
mod tests {
    use crate::parser::{Scene, parse_scenes};

    #[test]
    fn parse_results() {
        let results = "
Benchmark Summary (9.4.0 dev)
Name, Avg. CPU, Avg. FPS, Avg. time, render time, flush time
Empty screen, 35%, 56, 4, 0, 4
Moving wallpaper, 43%, 61, 6, 0, 6
Single rectangle, 55%, 61, 8, 0, 8
Multiple rectangles, 57%, 61, 8, 0, 8
Multiple RGB images, 60%, 61, 8, 0, 8
Multiple ARGB images, 74%, 61, 11, 3, 8
Rotated ARGB images, 76%, 61, 11, 3, 8
Multiple labels, 72%, 61, 10, 2, 8
Screen sized text, 80%, 60, 14, 5, 9
Multiple arcs, 72%, 61, 9, 1, 8
Containers, 63%, 61, 9, 1, 8
Containers with overlay, 88%, 58, 14, 7, 7
Containers with opa, 71%, 61, 9, 1, 8
Containers with opa_layer, 76%, 61, 11, 3, 8
Containers with scrolling, 75%, 61, 11, 4, 7
Widgets demo, 32%, 61, 11, 2, 9
All scenes avg.,64%, 60, 9, 2, 7
";

        let actual = parse_scenes(results).expect("Failed to parse result");
        let expected = vec![
            Scene {
                scene_name: "Empty screen".to_string(),
                avg_cpu: 35,
                avg_fps: 56,
                avg_time: 4,
                render_time: 0,
                flush_time: 4,
            },
            Scene {
                scene_name: "Moving wallpaper".to_string(),
                avg_cpu: 43,
                avg_fps: 61,
                avg_time: 6,
                render_time: 0,
                flush_time: 6,
            },
            Scene {
                scene_name: "Single rectangle".to_string(),
                avg_cpu: 55,
                avg_fps: 61,
                avg_time: 8,
                render_time: 0,
                flush_time: 8,
            },
            Scene {
                scene_name: "Multiple rectangles".to_string(),
                avg_cpu: 57,
                avg_fps: 61,
                avg_time: 8,
                render_time: 0,
                flush_time: 8,
            },
            Scene {
                scene_name: "Multiple RGB images".to_string(),
                avg_cpu: 60,
                avg_fps: 61,
                avg_time: 8,
                render_time: 0,
                flush_time: 8,
            },
            Scene {
                scene_name: "Multiple ARGB images".to_string(),
                avg_cpu: 74,
                avg_fps: 61,
                avg_time: 11,
                render_time: 3,
                flush_time: 8,
            },
            Scene {
                scene_name: "Rotated ARGB images".to_string(),
                avg_cpu: 76,
                avg_fps: 61,
                avg_time: 11,
                render_time: 3,
                flush_time: 8,
            },
            Scene {
                scene_name: "Multiple labels".to_string(),
                avg_cpu: 72,
                avg_fps: 61,
                avg_time: 10,
                render_time: 2,
                flush_time: 8,
            },
            Scene {
                scene_name: "Screen sized text".to_string(),
                avg_cpu: 80,
                avg_fps: 60,
                avg_time: 14,
                render_time: 5,
                flush_time: 9,
            },
            Scene {
                scene_name: "Multiple arcs".to_string(),
                avg_cpu: 72,
                avg_fps: 61,
                avg_time: 9,
                render_time: 1,
                flush_time: 8,
            },
            Scene {
                scene_name: "Containers".to_string(),
                avg_cpu: 63,
                avg_fps: 61,
                avg_time: 9,
                render_time: 1,
                flush_time: 8,
            },
            Scene {
                scene_name: "Containers with overlay".to_string(),
                avg_cpu: 88,
                avg_fps: 58,
                avg_time: 14,
                render_time: 7,
                flush_time: 7,
            },
            Scene {
                scene_name: "Containers with opa".to_string(),
                avg_cpu: 71,
                avg_fps: 61,
                avg_time: 9,
                render_time: 1,
                flush_time: 8,
            },
            Scene {
                scene_name: "Containers with opa_layer".to_string(),
                avg_cpu: 76,
                avg_fps: 61,
                avg_time: 11,
                render_time: 3,
                flush_time: 8,
            },
            Scene {
                scene_name: "Containers with scrolling".to_string(),
                avg_cpu: 75,
                avg_fps: 61,
                avg_time: 11,
                render_time: 4,
                flush_time: 7,
            },
            Scene {
                scene_name: "Widgets demo".to_string(),
                avg_cpu: 32,
                avg_fps: 61,
                avg_time: 11,
                render_time: 2,
                flush_time: 9,
            },
            Scene {
                scene_name: "All scenes avg.".to_string(),
                avg_cpu: 64,
                avg_fps: 60,
                avg_time: 9,
                render_time: 2,
                flush_time: 7,
            },
        ];
        assert_eq!(actual, expected);
    }
}
