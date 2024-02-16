use bevy::{prelude::*, utils::HashSet, window::PrimaryWindow};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::{prelude::*, dynamics::RopeJointBuilder};

const JUMP_GRACE_PERIOD : f32 = 0.1;

#[derive(Clone, Default, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderMassProperties,
}

#[derive(Clone, Default, Component)]
pub struct JumpComponent {
    pub on_ground: bool,
    pub last_on_ground: Option<f32>,
    pub last_tried_to_jump: Option<f32>,
    pub jumping: bool,
    pub last_time_jumped: Option<f32>,
    pub falling: bool
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    player: Player,
    #[sprite_bundle("player.png")]
    sprite_bundle: SpriteBundle,
    //#[grid_coords]
    //grid_coords: GridCoords,
    #[from_entity_instance]
    collider_bundle: ColliderBundle,
    jump_component: JumpComponent,
    #[worldly]
    worldy: Worldly
}

#[derive(Component)]
pub struct GroundSensor {
    pub ground_detection_entity: Entity,
    pub intersecting_ground_entities: HashSet<Entity>,
}

#[derive(Default, Component)]
pub struct Player;

#[derive(Default, Component)]
pub struct Grapple;

#[derive(Default, Bundle)]
pub struct GrapppleBundle {
    grapple: Grapple,
    sprite_bundle: SpriteBundle,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
}

impl From<&EntityInstance> for ColliderBundle {
    fn from(_value: &EntityInstance) -> Self {
        ColliderBundle {
            collider: Collider::cuboid(8.0, 8.0),
            rigid_body: RigidBody::Dynamic,
            friction: Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min
            },
            rotation_constraints: LockedAxes::ROTATION_LOCKED,
            ..Default::default()
        }
    }
}

pub fn movement(input: Res<Input<KeyCode>>, mut query: Query<(&mut Velocity, &mut JumpComponent, &mut GravityScale), With<Player>>, time: Res<Time>) {
    for (mut velocity, mut jump_component, mut gravity_scale) in &mut query {
        let right = if input.pressed(KeyCode::D) { 1.0 } else { 0.0 };
        let left = if input.pressed(KeyCode::A) { 1.0 } else { 0.0 };

        let target_speed : f32 = (right - left) * 90.0;
        let speed_difference : f32 = target_speed - velocity.linvel.x;
        let acceleration_rate = if target_speed.abs() > 0.01 { 0.1 } else { 0.15 };
        //let force = (speed_difference.abs() * acceleration_rate).powi(2) * speed_difference.signum();
        velocity.linvel.x += speed_difference * acceleration_rate;

        if let Some(last_time_jumped) = jump_component.last_time_jumped {
            if jump_component.jumping && (input.just_released(KeyCode::Space) || time.elapsed_seconds() - last_time_jumped > 0.5) {
                /*
                if velocity.linvel.y > 0.0 {
                    velocity.linvel.y /= 2.0;
                }*/

                jump_component.jumping = false;
                jump_component.falling = true;
                *gravity_scale = GravityScale(1.0);
            }
        }

        if input.just_pressed(KeyCode::Space) {
            jump_component.last_tried_to_jump = Some(time.elapsed_seconds());
        }

        if jump_component.on_ground {
            jump_component.last_on_ground = Some(time.elapsed_seconds());
            jump_component.jumping = false;
            jump_component.falling = false;
            *gravity_scale = GravityScale(1.0);
        }

        if let (Some(last_on_ground), Some(last_tried_to_jump)) = (jump_component.last_on_ground, jump_component.last_tried_to_jump) {
            if time.elapsed_seconds() - last_on_ground <= JUMP_GRACE_PERIOD &&
                time.elapsed_seconds() - last_tried_to_jump <= JUMP_GRACE_PERIOD {
                velocity.linvel.y = 70.0;
                jump_component.last_on_ground = None;
                jump_component.on_ground = false;
                jump_component.last_tried_to_jump = None;
                jump_component.jumping = true;
                jump_component.last_time_jumped = Some(time.elapsed_seconds());
                *gravity_scale = GravityScale(0.15);
            }
        }
    }
}

pub fn grapple(
    mut commands: Commands,
    mut player: Query<(Entity, &Transform, &mut Velocity), With<Player>>,
    grapple: Query<Entity, With<Grapple>>,
    input: Res<Input<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    rapier_context: Res<RapierContext>
) {
    if input.just_pressed(MouseButton::Left) {

        let (player_entity, player_transform, _) = player.single();
        let (camera, camera_transform) = camera.single();
        let window = window.single();
        if let Some(cursor_pos) = window.cursor_position() {
            if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                let ray_dir = (world_pos - player_transform.translation.xy()).normalize();
                if let Some((_, toi)) = rapier_context.cast_ray(player_transform.translation.xy(), ray_dir, 80.0, true, QueryFilter::only_fixed()) {
                    let hit_point = player_transform.translation.xy() + ray_dir * toi;

                    let joint = RopeJointBuilder::new(80.0)
                        .local_anchor1(Vec2::new(0.0, 0.0))
                        .local_anchor2(Vec2::new(0.0, 0.0)).build();

                    commands.spawn(GrapppleBundle {
                        sprite_bundle: SpriteBundle {
                            sprite: Sprite {
                                color: Color::RED,
                                custom_size: Some(Vec2::new(50.0, 1.0)),
                                anchor: bevy::sprite::Anchor::CenterLeft,
                                ..default()
                            },
                            transform: Transform::from_translation(Vec3::new(hit_point.x, hit_point.y, 10.0)),
                            ..default()
                        },
                        rigid_body: RigidBody::Dynamic,
                        gravity_scale: GravityScale(0.0),
                        ..default()
                    }).insert(ImpulseJoint::new(player_entity, joint));
                }
            }
        }
    }

    if input.just_released(MouseButton::Left) {
        if let Ok(entity) = grapple.get_single() {
            commands.entity(entity).despawn();

            let (_, _, mut player_velocity) = player.single_mut();
            if player_velocity.linvel.y > 0.0 {
                player_velocity.linvel.y *= 1.5;
            }
        }
    }
}

pub fn grapple_look_at_player(
    player: Query<&Transform, With<Player>>,
    mut grapple: Query<(&mut Transform, &mut Sprite), (With<Grapple>, Without<Player>)>
) {
    if let Ok(player) = player.get_single() {
        if let Ok((mut grapple, mut sprite)) = grapple.get_single_mut() {
            let diff = player.translation - grapple.translation;
            let angle = diff.y.atan2(diff.x);
            grapple.rotation = Quat::from_axis_angle(Vec3::Z, angle);
            sprite.custom_size = Some(Vec2::new(diff.length(), 1.0));
        }
    }
}

pub fn grapple_pull_player(
    mut player: Query<(&Transform, &mut Velocity), With<Player>>,
    grapple: Query<&Transform, With<Grapple>>
) {
    if let Ok((transform, mut velocity)) = player.get_single_mut() {
        if let Ok(grapple_transform) = grapple.get_single() {
            velocity.linvel += (grapple_transform.translation - transform.translation).xy() / 10.0;
        }
    }
}

pub fn spawn_ground_sensor(
    mut commands: Commands,
    detect_ground_for: Query<(Entity, &Collider), Added<JumpComponent>>,
) {
    for (entity, shape) in &detect_ground_for {
        if let Some(cuboid) = shape.as_cuboid() {
            let Vec2 {
                x: half_extents_x,
                y: half_extents_y,
            } = cuboid.half_extents();

            let detector_shape = Collider::cuboid(half_extents_x / 2.0, 2.);

            let sensor_translation = Vec3::new(0., -half_extents_y, 0.);

            commands.entity(entity).with_children(|builder| {
                builder
                    .spawn_empty()
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(detector_shape)
                    .insert(Sensor)
                    .insert(Transform::from_translation(sensor_translation))
                    .insert(GlobalTransform::default())
                    .insert(GroundSensor {
                        ground_detection_entity: entity,
                        intersecting_ground_entities: HashSet::new(),
                    });
            });
        }
    }
}

pub fn ground_detection(
    mut ground_sensors: Query<&mut GroundSensor>,
    mut collisions: EventReader<CollisionEvent>,
    collidables: Query<With<Collider>, Without<Sensor>>,
) {
    for collision_event in collisions.read() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                if collidables.contains(*e1) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e2) {
                        sensor.intersecting_ground_entities.insert(*e1);
                    }
                } else if collidables.contains(*e2) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e1) {
                        sensor.intersecting_ground_entities.insert(*e2);
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                if collidables.contains(*e1) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e2) {
                        sensor.intersecting_ground_entities.remove(e1);
                    }
                } else if collidables.contains(*e2) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e1) {
                        sensor.intersecting_ground_entities.remove(e2);
                    }
                }
            }
        }
    }
}

pub fn update_on_ground(
    mut ground_detectors: Query<&mut JumpComponent>,
    ground_sensors: Query<&GroundSensor, Changed<GroundSensor>>,
) {
    for sensor in &ground_sensors {
        if let Ok(mut ground_detection) = ground_detectors.get_mut(sensor.ground_detection_entity) {
            ground_detection.on_ground = !sensor.intersecting_ground_entities.is_empty();
        }
    }
}
