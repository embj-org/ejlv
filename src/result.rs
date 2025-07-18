use ej_config::ej_board_config::EjBoardConfigApi;
use tracing::warn;

use crate::scene::Scene;

pub fn calculate_result_delta(
    new_results: Vec<(EjBoardConfigApi, Vec<Scene>)>,
    previous_results: &Vec<(EjBoardConfigApi, Vec<Scene>)>,
) -> Vec<(EjBoardConfigApi, Vec<Scene>, Vec<Scene>)> {
    let mut result = Vec::new();
    for (new_config, new_result) in new_results.into_iter() {
        if let Some((_, prev_result)) = previous_results
            .iter()
            .find(|(prev_board_config, _)| prev_board_config.id == new_config.id)
        {
            let delta = calculate_delta(&new_result, prev_result);
            result.push((new_config, new_result, delta));
        } else {
            result.push((new_config, new_result.clone(), new_result));
        }
    }
    result
}
fn calculate_delta(a: &Vec<Scene>, b: &Vec<Scene>) -> Vec<Scene> {
    let mut result = Vec::new();
    for a_scene in a.iter() {
        if let Some(b_scene) = b
            .iter()
            .find(|b_scene| b_scene.scene_name == a_scene.scene_name)
        {
            result.push(Scene {
                scene_name: a_scene.scene_name.clone(),
                avg_cpu: a_scene.avg_cpu - b_scene.avg_cpu,
                avg_fps: a_scene.avg_fps - b_scene.avg_fps,
                avg_time: a_scene.avg_time - b_scene.avg_time,
                render_time: a_scene.render_time - b_scene.render_time,
                flush_time: a_scene.flush_time - b_scene.flush_time,
            });
        } else {
            warn!("Couldn't find scene '{}' in {:?}", a_scene.scene_name, b);
            result.push(a_scene.clone());
        }
    }
    for b_scene in b.iter() {
        if let None = a
            .iter()
            .find(|a_scene| a_scene.scene_name == b_scene.scene_name)
        {
            warn!("Couldn't find scene '{}' in {:?}", b_scene.scene_name, a);
            result.push(b_scene.clone());
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Helper function to create a test scene
    fn create_scene(name: &str, cpu: i32, fps: i32, time: i32, render: i32, flush: i32) -> Scene {
        Scene {
            scene_name: name.to_string(),
            avg_cpu: cpu,
            avg_fps: fps,
            avg_time: time,
            render_time: render,
            flush_time: flush,
        }
    }

    // Helper function to create a test config
    fn create_config(id: Uuid, name: &str, tags: Vec<&str>) -> EjBoardConfigApi {
        EjBoardConfigApi {
            id,
            name: name.to_string(),
            tags: tags.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_calculate_delta_identical_scenes() {
        let scene1 = create_scene("test_scene", 50, 60, 100, 80, 20);
        let scene2 = create_scene("test_scene", 50, 60, 100, 80, 20);

        let a = vec![scene1];
        let b = vec![scene2];

        let result = calculate_delta(&a, &b);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scene_name, "test_scene");
        assert_eq!(result[0].avg_cpu, 0);
        assert_eq!(result[0].avg_fps, 0);
        assert_eq!(result[0].avg_time, 0);
        assert_eq!(result[0].render_time, 0);
        assert_eq!(result[0].flush_time, 0);
    }

    #[test]
    fn test_calculate_delta_negative_values() {
        let curr = create_scene("test_scene", 40, 55, 90, 70, 15);
        let prev = create_scene("test_scene", 50, 60, 100, 80, 20);

        let curr = vec![curr];
        let prev = vec![prev];

        let result = calculate_delta(&curr, &prev);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scene_name, "test_scene");
        assert_eq!(result[0].avg_cpu, -10);
        assert_eq!(result[0].avg_fps, -5);
        assert_eq!(result[0].avg_time, -10);
        assert_eq!(result[0].render_time, -10);
        assert_eq!(result[0].flush_time, -5);
    }

    #[test]
    fn test_calculate_delta_different_values() {
        let curr = create_scene("test_scene", 60, 70, 110, 90, 25);
        let prev = create_scene("test_scene", 50, 60, 100, 80, 20);

        let curr = vec![curr];
        let prev = vec![prev];

        let result = calculate_delta(&curr, &prev);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scene_name, "test_scene");
        assert_eq!(result[0].avg_cpu, 10);
        assert_eq!(result[0].avg_fps, 10);
        assert_eq!(result[0].avg_time, 10);
        assert_eq!(result[0].render_time, 10);
        assert_eq!(result[0].flush_time, 5);
    }

    #[test]
    fn test_calculate_delta_scene_only_in_a() {
        let scene_a = create_scene("unique_scene", 40, 55, 90, 70, 15);
        let scene_b = create_scene("other_scene", 50, 60, 100, 80, 20);

        let a = vec![scene_a.clone()];
        let b = vec![scene_b];

        let result = calculate_delta(&a, &b);

        assert_eq!(result.len(), 2);

        // The unique scene from 'a' should be cloned as-is
        let unique_scene = result
            .iter()
            .find(|s| s.scene_name == "unique_scene")
            .unwrap();
        assert_eq!(unique_scene.avg_cpu, 40);
        assert_eq!(unique_scene.avg_fps, 55);

        // The scene from 'b' should be added
        let other_scene = result
            .iter()
            .find(|s| s.scene_name == "other_scene")
            .unwrap();
        assert_eq!(other_scene.avg_cpu, 50);
        assert_eq!(other_scene.avg_fps, 60);
    }

    #[test]
    fn test_calculate_delta_scene_only_in_b() {
        let scene_a = create_scene("common_scene", 40, 55, 90, 70, 15);
        let scene_b1 = create_scene("common_scene", 50, 60, 100, 80, 20);
        let scene_b2 = create_scene("unique_in_b", 30, 45, 85, 65, 10);

        let a = vec![scene_a];
        let b = vec![scene_b1, scene_b2.clone()];

        let result = calculate_delta(&a, &b);

        assert_eq!(result.len(), 2);

        // The common scene should have delta calculated
        let common_scene = result
            .iter()
            .find(|s| s.scene_name == "common_scene")
            .unwrap();
        assert_eq!(common_scene.avg_cpu, -10);

        // The unique scene from 'b' should be cloned as-is
        let unique_scene = result
            .iter()
            .find(|s| s.scene_name == "unique_in_b")
            .unwrap();
        assert_eq!(unique_scene.avg_cpu, 30);
        assert_eq!(unique_scene.avg_fps, 45);
    }

    #[test]
    fn test_calculate_delta_multiple_scenes() {
        let scenes_a = vec![
            create_scene("scene1", 40, 55, 90, 70, 15),
            create_scene("scene2", 35, 50, 85, 65, 12),
        ];

        let scenes_b = vec![
            create_scene("scene1", 50, 60, 100, 80, 20),
            create_scene("scene2", 45, 55, 95, 75, 18),
        ];

        let result = calculate_delta(&scenes_a, &scenes_b);

        assert_eq!(result.len(), 2);

        let scene1_result = result.iter().find(|s| s.scene_name == "scene1").unwrap();
        assert_eq!(scene1_result.avg_cpu, -10);
        assert_eq!(scene1_result.avg_fps, -5);

        let scene2_result = result.iter().find(|s| s.scene_name == "scene2").unwrap();
        assert_eq!(scene2_result.avg_cpu, -10);
        assert_eq!(scene2_result.avg_fps, -5);
    }

    #[test]
    fn test_calculate_delta_empty_vectors() {
        let empty_a: Vec<Scene> = vec![];
        let empty_b: Vec<Scene> = vec![];

        let result = calculate_delta(&empty_a, &empty_b);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_calculate_result_delta_matching_configs() {
        let config_id = Uuid::new_v4();
        let config = create_config(config_id, "test_config", vec!["tag1"]);

        let new_scene = create_scene("test_scene", 50, 60, 100, 80, 20);
        let prev_scene = create_scene("test_scene", 40, 55, 90, 70, 15);

        let new_results = vec![(config.clone(), vec![new_scene])];
        let previous_results = vec![(config.clone(), vec![prev_scene])];

        let result = calculate_result_delta(new_results, &previous_results);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0.id, config_id);
        assert_eq!(result[0].1.len(), 1); // new_result
        assert_eq!(result[0].2.len(), 1); // delta

        // Check delta calculation
        let delta_scene = &result[0].2[0];
        assert_eq!(delta_scene.avg_cpu, 10); // 50 - 40
        assert_eq!(delta_scene.avg_fps, 5); // 60 - 55
    }

    #[test]
    fn test_calculate_result_delta_no_previous_config() {
        let config_id = Uuid::new_v4();
        let config = create_config(config_id, "new_config", vec!["tag1"]);

        let new_scene = create_scene("test_scene", 50, 60, 100, 80, 20);

        let new_results = vec![(config.clone(), vec![new_scene.clone()])];
        let previous_results = vec![];

        let result = calculate_result_delta(new_results, &previous_results);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0.id, config_id);
        assert_eq!(result[0].1.len(), 1); // new_result
        assert_eq!(result[0].2.len(), 1); // delta should be same as new_result

        // When no previous config exists, delta should be identical to new_result
        assert_eq!(result[0].1[0].avg_cpu, result[0].2[0].avg_cpu);
        assert_eq!(result[0].1[0].avg_fps, result[0].2[0].avg_fps);
    }

    #[test]
    fn test_calculate_result_delta_different_configs() {
        let config1_id = Uuid::new_v4();
        let config2_id = Uuid::new_v4();
        let config1 = create_config(config1_id, "config1", vec!["tag1"]);
        let config2 = create_config(config2_id, "config2", vec!["tag2"]);

        let new_scene = create_scene("test_scene", 50, 60, 100, 80, 20);
        let prev_scene = create_scene("test_scene", 40, 55, 90, 70, 15);

        let new_results = vec![(config1.clone(), vec![new_scene.clone()])];
        let previous_results = vec![(config2.clone(), vec![prev_scene])];

        let result = calculate_result_delta(new_results, &previous_results);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0.id, config1_id);

        // Since config IDs don't match, delta should be same as new_result
        assert_eq!(result[0].1[0].avg_cpu, result[0].2[0].avg_cpu);
        assert_eq!(result[0].1[0].avg_fps, result[0].2[0].avg_fps);
    }

    #[test]
    fn test_calculate_result_delta_multiple_configs() {
        let config1_id = Uuid::new_v4();
        let config2_id = Uuid::new_v4();
        let config1 = create_config(config1_id, "config1", vec!["tag1"]);
        let config2 = create_config(config2_id, "config2", vec!["tag2"]);

        let new_scene1 = create_scene("scene1", 50, 60, 100, 80, 20);
        let new_scene2 = create_scene("scene2", 45, 55, 95, 75, 18);
        let prev_scene1 = create_scene("scene1", 40, 55, 90, 70, 15);
        let prev_scene2 = create_scene("scene2", 35, 50, 85, 65, 12);

        let new_results = vec![
            (config1.clone(), vec![new_scene1]),
            (config2.clone(), vec![new_scene2]),
        ];
        let previous_results = vec![
            (config1.clone(), vec![prev_scene1]),
            (config2.clone(), vec![prev_scene2]),
        ];

        let result = calculate_result_delta(new_results, &previous_results);

        assert_eq!(result.len(), 2);

        // Both configs should have deltas calculated
        let result1 = result.iter().find(|r| r.0.id == config1_id).unwrap();
        let result2 = result.iter().find(|r| r.0.id == config2_id).unwrap();

        assert_eq!(result1.2[0].avg_cpu, 10); // 50 - 40
        assert_eq!(result2.2[0].avg_cpu, 10); // 45 - 35
    }

    #[test]
    fn test_calculate_result_delta_empty_inputs() {
        let new_results = vec![];
        let previous_results = vec![];

        let result = calculate_result_delta(new_results, &previous_results);

        assert_eq!(result.len(), 0);
    }
}
