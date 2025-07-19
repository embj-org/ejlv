use plotters::{
    chart::ChartBuilder,
    coord::Shift,
    prelude::{DrawingArea, PathElement, Rectangle, SVGBackend, Text},
    style::{Color, RGBColor},
};

use crate::{
    prelude::*,
    scene::{Scene, SceneMetric},
};
pub struct RunResult {
    run_name: String,
    scenes: Vec<Scene>,
}

impl RunResult {
    pub fn new(name: impl Into<String>, results: Vec<Scene>) -> Self {
        Self {
            run_name: name.into(),
            scenes: results,
        }
    }
}
pub static COLORS: [RGBColor; 9] = [
    RGBColor(220, 80, 80),   // Muted Red
    RGBColor(100, 180, 100), // Muted Green
    RGBColor(100, 150, 230), // Muted Blue
    RGBColor(240, 210, 90),  // Warm Yellow
    RGBColor(100, 200, 200), // Muted Cyan
    RGBColor(200, 100, 200), // Soft Magenta
    RGBColor(255, 160, 90),  // Tangerine
    RGBColor(90, 180, 180),  // Teal
    RGBColor(180, 140, 210), // Lilac
];

fn get_max_value(scenes: &Vec<Scene>, metric: &SceneMetric) -> i32 {
    let mut max = 0;

    for scene in scenes {
        let value = scene.get_value(&metric);
        if value > max {
            max = value;
        }
    }
    return max;
}

pub fn create_comparison_chart(
    root: &DrawingArea<SVGBackend<'_>, Shift>,
    results: &[RunResult],
    metric: &SceneMetric,
    colors: &[RGBColor],
) -> Result<()> {
    if results.is_empty() {
        return Err(Error::ResultSliceEmpty);
    }

    let mut max_value = 0;
    for result in results {
        let value = get_max_value(&result.scenes, &metric);
        if value > max_value {
            max_value = value;
        }
    }
    let scene_count = results[0].scenes.len();
    let bar_width = 2.;
    let bar_gap = 0.;
    let scene_gap = 1.;
    let bars_per_scene = results.len();
    let total_scene_width = bars_per_scene as f32 * (bar_width + bar_gap);
    let total_width_per_scene = total_scene_width + scene_gap;

    let run_names: Vec<&str> = results.iter().map(|r| r.run_name.as_str()).collect();
    let mut chart = ChartBuilder::on(&root)
        .caption(
            &format!("{} Comparison [{}]", metric.label(), run_names.join(", ")),
            ("sans-serif", 20),
        )
        .margin(10)
        .x_label_area_size(50)
        .y_label_area_size(10)
        .build_cartesian_2d(
            0f32..max_value as f32 * 1.2,
            0f32..(scene_count as f32 * total_width_per_scene),
        )?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_desc(metric.label())
        .y_desc("Scenes")
        .y_labels(0)
        .x_label_offset(total_width_per_scene)
        .draw()?;

    for (result_i, (result, color)) in results.iter().zip(colors).enumerate() {
        let values: Vec<i32> = result
            .scenes
            .iter()
            .map(|scene| scene.get_value(&metric))
            .collect();

        chart
            .draw_series(values.iter().enumerate().map(|(i, &value)| {
                let scene_start_y = i as f32 * total_width_per_scene;
                let bar_offset = result_i as f32 * (bar_width + bar_gap);
                let y0 = scene_start_y + bar_offset;
                let y1 = y0 + bar_width;
                Rectangle::new([(0.0, y0), (value as f32, y1)], color.filled())
            }))?
            .label(result.run_name.clone())
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], *color));

        chart.draw_series(values.iter().enumerate().map(|(i, scene)| {
            let scene_start_y = i as f32 * total_width_per_scene;
            let bar_offset = result_i as f32 * (bar_width + bar_gap);
            let y0 = scene_start_y + bar_offset + bar_width;
            Text::new(scene.to_string(), (0.0, y0), ("sans-serif", 12))
        }))?;
    }
    chart.draw_series(results[0].scenes.iter().enumerate().map(|(i, scene)| {
        let scene_center_y = i as f32 * total_width_per_scene + total_scene_width / 2.0;
        let max_bar_height = results
            .iter()
            .map(|r| r.scenes[i].get_value(&metric))
            .max()
            .unwrap_or(0) as f32;
        Text::new(
            scene.scene_name.clone(),
            (max_bar_height + 10.0, scene_center_y),
            ("sans-serif", 10),
        )
    }))?;

    chart.configure_series_labels().draw()?;
    Ok(())
}
