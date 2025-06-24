use bevy::prelude::*;
use bevy_mod_outline::*;

use std::f32::consts::PI;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);
}

const TANK_WIDTH: f32 = 2.0;
const TANK_LENGTH: f32 = 5.0;
const TANK_HEIGHT: f32 = 0.75;
const TANK_TURRET_HEIGHT: f32 = 0.5;
const TANK_TURRET_RADIUS: f32 = 0.75;
const TANK_GUN_LENGTH: f32 = 3.0;
const TANK_GUN_RADIUS: f32 = 0.05;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let token_assets = TokenAssets {
        shape: meshes.add(Cuboid::new(TANK_WIDTH, TANK_HEIGHT, TANK_LENGTH)),
        turret_shape: meshes.add(Extrusion::new(
            Circle::new(TANK_TURRET_RADIUS),
            TANK_TURRET_HEIGHT,
        )),
        gun_shape: meshes.add(Cylinder::new(TANK_GUN_RADIUS, TANK_GUN_LENGTH)),
        material: materials.add(StandardMaterial::from_color(Color::srgb_u8(255, 0, 0))),
    };

    commands.insert_resource(token_assets);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(super) struct Selected;

#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub(super) struct TokenAssets {
    shape: Handle<Mesh>,
    turret_shape: Handle<Mesh>,
    gun_shape: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

pub(super) fn spawn_token_on_click(
    _event: Trigger<Pointer<Pressed>>,
    mut commands: Commands,
    token_assets: Res<TokenAssets>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if !keys.pressed(KeyCode::ControlLeft) && !keys.pressed(KeyCode::ControlRight) {
        return;
    }

    let (camera, camera_transform) = *camera;

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            if let Some(distance) =
                ray.intersect_plane(Vec3::new(0.0, 0.0, 0.0), InfinitePlane3d::new(Vec3::Y))
            {
                let point = ray.get_point(distance);
                spawn_token(&mut commands, point, &token_assets);
            }
        }
    }
}

fn spawn_token(commands: &mut Commands, position: Vec3, token_assets: &TokenAssets) {
    commands
        .spawn((
            Mesh3d(token_assets.shape.clone()),
            MeshMaterial3d(token_assets.material.clone()),
            Transform::from_translation(position + Vec3::new(0.0, TANK_HEIGHT / 2.0, 0.0)),
            OutlineVolume {
                width: 4.0f32,
                ..default()
            },
            OutlineMode::FloodFlat,
            Pickable::default(),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Mesh3d(token_assets.turret_shape.clone()),
                    MeshMaterial3d(token_assets.material.clone()),
                    Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0)).with_translation(
                        Vec3::new(0.0, TANK_HEIGHT / 2.0 + TANK_TURRET_HEIGHT / 2.0, 0.0),
                    ),
                    Pickable::IGNORE,
                ))
                .with_children(|parent2| {
                    parent2.spawn((
                        Mesh3d(token_assets.gun_shape.clone()),
                        MeshMaterial3d(token_assets.material.clone()),
                        Transform::from_translation(Vec3::new(0.0, TANK_GUN_LENGTH / 2.0, 0.0)),
                        Pickable::IGNORE,
                    ));
                });
        })
        .observe(select_on_click)
        .observe(drag_on_drag);
}

pub(super) fn select_on_click(
    mut event: Trigger<Pointer<Pressed>>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut OutlineVolume, Option<&Selected>)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    event.propagate(false);

    fn deselect_all(
        commands: &mut Commands,
        query: &mut Query<(Entity, &mut OutlineVolume, Option<&Selected>)>,
    ) {
        for (entity, mut outline, selected) in query.iter_mut() {
            if selected.is_some() {
                if let Ok(mut entity) = commands.get_entity(entity) {
                    entity.remove::<Selected>();
                    outline.visible = false;
                }
            }
        }
    }

    let multi_select = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if !multi_select {
        deselect_all(&mut commands, &mut query);
    }

    if let Ok((entity, mut outline, selected)) = query.get_mut(event.target) {
        if let Ok(mut entity) = commands.get_entity(entity) {
            if multi_select && selected.is_some() {
                entity.remove::<Selected>();
                outline.visible = false;
            } else {
                entity.insert(Selected);
                outline.visible = true;
            }
        }
    }
}

fn drag_on_drag(
    drag: Trigger<Pointer<Drag>>,
    mut transforms: Query<&mut Transform>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    let (camera, camera_transform) = *camera;

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };

    let Some(distance) =
        ray.intersect_plane(Vec3::new(0.0, 0.0, 0.0), InfinitePlane3d::new(Vec3::Y))
    else {
        return;
    };
    let point = ray.get_point(distance);

    let Ok(prev_ray) = camera.viewport_to_world(camera_transform, cursor_pos - drag.delta) else {
        return;
    };

    let Some(prev_distance) =
        prev_ray.intersect_plane(Vec3::new(0.0, 0.0, 0.0), InfinitePlane3d::new(Vec3::Y))
    else {
        return;
    };
    let prev_point = prev_ray.get_point(prev_distance);

    let world_delta = point - prev_point;

    let mut transform = transforms.get_mut(drag.target).unwrap();
    transform.translation.x += world_delta.x;
    transform.translation.z += world_delta.z;
}
