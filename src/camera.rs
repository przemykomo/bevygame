use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_ecs_ldtk::prelude::*;

use crate::Player;

#[derive(Default, Component)]
pub struct CameraTransition {
    pub is_changing_level: bool,
    pub begin_position: Vec2,
    pub begin_time: f32,
    pub begin_scale: Vec2
}

#[derive(Default, Bundle)]
pub struct CustomCameraBundle {
    camera_2d_bundle: Camera2dBundle,
    transition: CameraTransition
}

const MAX_SCREEN_WIDTH : f32 = 350.0;

pub fn camera_fit_inside_current_level(
    mut camera_query: Query<
        (
            &mut OrthographicProjection,
            &mut Transform,
            &mut CameraTransition
        ),
        Without<Player>,
    >,
    player_query: Query<&Transform, With<Player>>,
    level_query: Query<(&Transform, &LevelIid), (Without<OrthographicProjection>, Without<Player>)>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    level_selection: Res<LevelSelection>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    window: Query<&Window>,
    time: Res<Time>
) {
    if let Ok(Transform {
        translation: player_translation,
        ..
    }) = player_query.get_single()
    {
        let player_translation = *player_translation;

        let (mut orthographic_projection, mut camera_transform, mut camera_transition) = camera_query.single_mut();
        let window = window.single();
        let aspect_ratio : f32 = window.width() / window.height();
        let mut wanted_camera_position = Vec2::new(0.0, 0.0);
        let mut wanted_scale = Vec2::new(0.0, 0.0);

        for (level_transform, level_iid) in &level_query {
            let ldtk_project = ldtk_project_assets
                .get(ldtk_projects.single())
                .expect("Project should be loaded if level has spawned");

            let level = ldtk_project
                .get_raw_level_by_iid(&level_iid.to_string())
                .expect("Spawned level should exist in LDtk project");

            if level_selection.is_match(&LevelIndices::default(), level) {
                let level_ratio = level.px_wid as f32 / level.px_hei as f32;
                orthographic_projection.viewport_origin = Vec2::ZERO;
                if level.px_wid as f32 >= MAX_SCREEN_WIDTH && level.px_hei as f32 >= MAX_SCREEN_WIDTH / aspect_ratio {
                    let height = MAX_SCREEN_WIDTH / aspect_ratio;
                    wanted_scale.x = MAX_SCREEN_WIDTH;
                    wanted_scale.y = height;
                    wanted_camera_position.x = (player_translation.x - MAX_SCREEN_WIDTH / 2.0).clamp(level_transform.translation.x, level_transform.translation.x + level.px_wid as f32 - MAX_SCREEN_WIDTH);
                    wanted_camera_position.y = (player_translation.y - height / 2.0).clamp(level_transform.translation.y, level_transform.translation.y + level.px_hei as f32 - height);
                } else if level_ratio > aspect_ratio {
                    // level is wider than the screen
                    //let height = (level.px_hei as f32 / 9.).round() * 9.;
                    let height = level.px_hei as f32;
                    let width = height * aspect_ratio;
                    wanted_scale.x = width;
                    wanted_scale.y = height;
                    wanted_camera_position.x = (player_translation.x - width / 2.0).clamp(level_transform.translation.x, level_transform.translation.x + level.px_wid as f32 - width);
                    wanted_camera_position.y = level_transform.translation.y;
                    /*
                    orthographic_projection.scaling_mode =
                        bevy::render::camera::ScalingMode::Fixed { width, height };
                    camera_transform.translation.x =
                        (player_translation.x - level_transform.translation.x - width / 2.)
                            .clamp(0., level.px_wid as f32 - width);
                    camera_transform.translation.y = 0.; */
                } else {
                    // level is taller than the screen
                    //let width = (level.px_wid as f32 / 16.).round() * 16.;
                    let width = level.px_wid as f32;
                    let height = width / aspect_ratio;
                    wanted_scale.x = width;
                    wanted_scale.y = height;
                    wanted_camera_position.y = (player_translation.y - height / 2.0).clamp(level_transform.translation.y, level_transform.translation.y + level.px_hei as f32 - height);
                    wanted_camera_position.x = level_transform.translation.x;
                    /*
                    let height = width / ASPECT_RATIO;
                    orthographic_projection.scaling_mode =
                        bevy::render::camera::ScalingMode::Fixed { width, height };
                    camera_transform.translation.y =
                        (player_translation.y - (level_transform.translation.y - height) / 2.)
                            .clamp(0., level.px_hei as f32 - height);
                    camera_transform.translation.x = 0.; */
                }

                //dbg!(camera_transform.translation, player_translation, level_transform.translation);
                //camera_transform.translation = player_translation;
/*
                camera_transform.translation.x += level_transform.translation.x;
                camera_transform.translation.y += level_transform.translation.y;*/
            }
        }

        if camera_transition.is_changing_level {
            let now = time.elapsed_seconds();

            if now - camera_transition.begin_time >= 0.5 {
                camera_transition.is_changing_level = false;
            }

            let lerped_pos = camera_transition.begin_position.lerp(wanted_camera_position, (now - camera_transition.begin_time) / 0.5);
            camera_transform.translation.x = lerped_pos.x;
            camera_transform.translation.y = lerped_pos.y;

            let lerped_scale = camera_transition.begin_scale.lerp(wanted_scale, (now - camera_transition.begin_time) / 0.5);
            orthographic_projection.scaling_mode = ScalingMode::Fixed { width: lerped_scale.x, height: lerped_scale.y };
        } else {
            camera_transform.translation.x = wanted_camera_position.x;
            camera_transform.translation.y = wanted_camera_position.y;
            orthographic_projection.scaling_mode = ScalingMode::Fixed { width: wanted_scale.x, height: wanted_scale.y };
        }
    }
}
