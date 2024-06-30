use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

#[derive(Default, Component)]
pub struct Wall;

#[derive(Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall
}

#[derive(Default, Component, Copy, Clone)]
pub struct Spikes;

#[derive(Default, Bundle, LdtkIntCell)]
pub struct SpikesBundle {
    spikes: Spikes
}


