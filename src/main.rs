pub mod player;
pub mod misc;
pub mod camera;
pub mod wall_collision;

use bevy::{log::LogPlugin, prelude::*};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use player::*;
use misc::*;
use camera::*;
use wall_collision::spawn_wall_collision;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())
            .set(LogPlugin {
            level: bevy::log::Level::INFO,
            ..default()
            })
        )
        .add_plugins(LdtkPlugin)
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation { load_level_neighbors: true },
            ..default()
        })
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .insert_resource(RapierConfiguration::new(20.4))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, camera_fit_inside_current_level)
        .add_systems(Update, level_selection_follow_player)
        .add_systems(Update, movement)
        .add_systems(Update, grapple)
        .add_systems(Update, grapple_look_at_player)
        //.add_systems(Update, grapple_pull_player)
        .add_systems(Update, spawn_wall_collision)
        .add_systems(Update, spawn_ground_sensor)
        .add_systems(Update, ground_detection)
        .add_systems(Update, update_on_ground)
        .add_systems(Update, collide_with_spikes)
        .insert_resource(LevelSelection::iid("0f72e230-b0a0-11ee-851b-03ba2455339d"))
        .register_ldtk_entity::<PlayerBundle>("Player")
        .register_ldtk_entity::<SpawnpointBundle>("Spawnpoint")
        .register_ldtk_entity::<HookBundle>("Hook")
        .register_ldtk_int_cell::<WallBundle>(1)
        .register_ldtk_int_cell::<WallBundle>(2)
        .register_ldtk_int_cell::<SpikesBundle>(4)
        //.add_plugins(WorldInspectorPlugin::new())
        .run();
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(CustomCameraBundle::default());

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("gamemap.ldtk"),
        ..Default::default()
    });
}

#[derive(Default, Component)]
pub struct Hook;

#[derive(Default, Bundle, LdtkEntity)]
pub struct HookBundle {
    hook: Hook,

    #[sprite_bundle("hook.png")]
    sprite_bundle: SpriteBundle,
}

#[derive(Default, Component)]
pub struct PushPlatform;

#[derive(Default, Bundle, LdtkEntity)]
pub struct PushPlatformBundle {
    hook: PushPlatform,

    #[sprite_bundle("push_platform.png")]
    sprite_bundle: SpriteBundle,
}
