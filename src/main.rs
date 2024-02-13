pub mod player;
pub mod setup;

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use player::*;
use setup::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(LdtkPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -200.0),
            ..Default::default()
        })
        //.add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, movement)
        .add_systems(Update, grapple)
        .add_systems(Update, grapple_look_at_player)
        .add_systems(Update, grapple_pull_player)
        .add_systems(Update, spawn_wall_collision)
        .add_systems(Update, camera_fit_inside_current_level)
        .add_systems(Update, spawn_ground_sensor)
        .add_systems(Update, ground_detection)
        .add_systems(Update, update_on_ground)
        .insert_resource(LevelSelection::index(0))
        .register_ldtk_entity::<PlayerBundle>("Player")
        .register_ldtk_entity::<GoalBundle>("Goal")
        .register_ldtk_int_cell::<WallBundle>(1)
        .run();
}

fn camera_fit_inside_current_level(
    mut camera_query: Query<
        (
            &mut bevy::render::camera::OrthographicProjection,
            &mut Transform,
        ),
        Without<Player>,
    >,
    player_query: Query<&Transform, With<Player>>,
    level_query: Query<(&Transform, &LevelIid), (Without<OrthographicProjection>, Without<Player>)>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    //level_selection: Res<LevelSelection>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    window: Query<&Window>
) {
    if let Ok(Transform {
        translation: player_translation,
        ..
    }) = player_query.get_single()
    {
        let player_translation = *player_translation;

        let (mut orthographic_projection, mut camera_transform) = camera_query.single_mut();
        let window = window.single();
        let aspect_ratio : f32 = window.width() / window.height();

        for (level_transform, level_iid) in &level_query {
            let ldtk_project = ldtk_project_assets
                .get(ldtk_projects.single())
                .expect("Project should be loaded if level has spawned");

            let level = ldtk_project
                .get_raw_level_by_iid(&level_iid.to_string())
                .expect("Spawned level should exist in LDtk project");

            //if level_selection.is_match(&LevelIndices::default(), level) {
                let level_ratio = level.px_wid as f32 / level.px_hei as f32;
                orthographic_projection.viewport_origin = Vec2::ZERO;
                if level_ratio > aspect_ratio {
                    // level is wider than the screen
                    //let height = (level.px_hei as f32 / 9.).round() * 9.;
                    let height = level.px_hei as f32;
                    let width = height * aspect_ratio;
                    orthographic_projection.scaling_mode = ScalingMode::FixedVertical(height);
                    camera_transform.translation.x = (player_translation.x - width / 2.0).clamp(0.0, level.px_wid as f32 - width);
                    camera_transform.translation.y = 0.0;
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
                    orthographic_projection.scaling_mode = ScalingMode::FixedHorizontal(width);
                    camera_transform.translation.y = (player_translation.y - height / 2.0).clamp(0.0, level.px_hei as f32 - height);
                    camera_transform.translation.x = 0.0;
                    /*
                    let height = width / ASPECT_RATIO;
                    orthographic_projection.scaling_mode =
                        bevy::render::camera::ScalingMode::Fixed { width, height };
                    camera_transform.translation.y =
                        (player_translation.y - (level_transform.translation.y - height) / 2.)
                            .clamp(0., level.px_hei as f32 - height);
                    camera_transform.translation.x = 0.; */
                }

                camera_transform.translation.x += level_transform.translation.x;
                camera_transform.translation.y += level_transform.translation.y;
            //}
        }
    }
}


#[derive(Default, Component)]
struct Goal;

#[derive(Default, Bundle, LdtkEntity)]
struct GoalBundle {
    goal: Goal,
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords
}

