use std::str::FromStr;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    pub scene_name: String,
    pub avg_cpu: i32,
    pub avg_fps: i32,
    pub avg_time: i32,
    pub render_time: i32,
    pub flush_time: i32,
}

#[derive(Debug, Clone)]
pub enum SceneMetric {
    FPS,
    CPU,
    AvgTime,
    RenderTime,
    FlushTime,
}

impl Scene {
    pub fn get_value(&self, metric: &SceneMetric) -> i32 {
        match metric {
            SceneMetric::FPS => self.avg_fps,
            SceneMetric::CPU => self.avg_cpu,
            SceneMetric::AvgTime => self.avg_time,
            SceneMetric::RenderTime => self.render_time,
            SceneMetric::FlushTime => self.flush_time,
        }
    }
}

impl SceneMetric {
    pub fn label(&self) -> &'static str {
        match self {
            SceneMetric::FPS => "FPS",
            SceneMetric::CPU => "CPU Usage (%)",
            SceneMetric::AvgTime => "Average Time (ms)",
            SceneMetric::RenderTime => "Render Time (ms)",
            SceneMetric::FlushTime => "Flush Time (ms)",
        }
    }

    pub fn snake_case(&self) -> &'static str {
        match self {
            SceneMetric::FPS => "fps",
            SceneMetric::CPU => "cpu",
            SceneMetric::AvgTime => "avg_time",
            SceneMetric::RenderTime => "render_time",
            SceneMetric::FlushTime => "flush_time",
        }
    }
}

impl FromStr for SceneMetric {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "fps" => Ok(SceneMetric::FPS),
            "cpu" => Ok(SceneMetric::CPU),
            "avg_time" => Ok(SceneMetric::AvgTime),
            "render_time" => Ok(SceneMetric::RenderTime),
            "flush_time" => Ok(SceneMetric::FlushTime),
            _ => Err(Error::InvalidMetric(s.to_string())),
        }
    }
}
