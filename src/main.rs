use bevy::prelude::*;
use bevy_mod_outline::*;
use std::f32::consts::PI;

/// Tag to track if an object is selected or not
#[derive(Component)]
struct Selected;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin, OutlinePlugin))
        .add_systems(Startup, (setup, spawn_initial_token).chain())
        .run();
}

#[derive(Resource, Clone)]
struct TokenAssets {
    shape: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Component)]
struct Ground;

fn spawn_token_on_click(
    _event: Trigger<Pointer<Pressed>>,
    mut commands: Commands,
    token_assets: Res<TokenAssets>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    ground: Single<&GlobalTransform, With<Ground>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if !keys.pressed(KeyCode::ControlLeft) && !keys.pressed(KeyCode::ControlRight) {
        return;
    }

    let (camera, camera_transform) = *camera;

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            if let Some(distance) =
                ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
            {
                let point = ray.get_point(distance);
                spawn_token(&mut commands, point, &token_assets);
            }
        }
    }
}

const TOKEN_HEIGHT: f32 = 0.1;
const TOKEN_RADIUS: f32 = 0.5;

fn spawn_token(commands: &mut Commands, position: Vec3, token_assets: &TokenAssets) {
    // Spawn token
    commands
        .spawn((
            Mesh3d(token_assets.shape.clone()),
            MeshMaterial3d(token_assets.material.clone()),
            Transform::from_translation(position + Vec3::new(0.0, TOKEN_HEIGHT / 2.0, 0.0))
                .with_rotation(Quat::from_rotation_x(-PI / 2.0)),
            OutlineVolume {
                width: 4.0f32,
                ..default()
            },
            OutlineMode::FloodFlat,
            Pickable::default(),
        ))
        .observe(select_on_click)
        .observe(drag_on_drag);
}

// System to spawn the initial token after setup
fn spawn_initial_token(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let token_assets = TokenAssets {
        shape: meshes.add(Extrusion::new(Circle::new(TOKEN_RADIUS), TOKEN_HEIGHT)),
        material: materials.add(StandardMaterial::from_color(Color::srgb_u8(255, 0, 0))),
    };

    // Clone the token_assets before moving it into the resource
    let token_assets_clone = token_assets.clone();
    commands.insert_resource(token_assets);

    let position = Vec3::new(0.0, 0.0, 0.05); // ground_height (0.1) / 2.0
    spawn_token(&mut commands, position, &token_assets_clone);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut window: Query<Entity, With<Window>>,
) {
    // Setup ground
    let ground_height = 0.1;
    let ground_shape = meshes.add(Plane3d::default().mesh().size(20.0, 20.0));
    let ground_material = materials.add(StandardMaterial::from_color(Color::WHITE));

    // Add an observer to the window to respond to clicks that don't hit a mesh
    if let Ok(entity) = window.single_mut() {
        if let Ok(mut window) = commands.get_entity(entity) {
            window.observe(select_on_click);
        }
    }

    // Add ground
    commands
        .spawn((
            Mesh3d(ground_shape.clone()),
            MeshMaterial3d(ground_material.clone()),
            Transform::from_translation(Vec3::new(0.0, -ground_height, 0.0)),
            Ground,
            Pickable::default(),
        ))
        .observe(spawn_token_on_click);

    // Add a light source
    commands.spawn((
        PointLight {
            color: Color::srgb_u8(255, 255, 192),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Add camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Observer system that manages what pickable objects are selected
fn select_on_click(
    mut event: Trigger<Pointer<Pressed>>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut OutlineVolume, Option<&Selected>)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Disable propagation to prevent observers other than the first in the queue from responding
    event.propagate(false);

    /// Helper function to deselect every selected entity
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

    // Check if the user wants to select multiple objects
    let multi_select = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    // Deselect everything if the user does not want to select multiple objects
    if !multi_select {
        deselect_all(&mut commands, &mut query);
    }

    // Select a mesh
    if let Ok((entity, mut outline, selected)) = query.get_mut(event.target) {
        if let Ok(mut entity) = commands.get_entity(entity) {
            // When selecting multiple objects, allow clicking a selected object to deselect it.
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
    ground: Single<&GlobalTransform, With<Ground>>,
) {
    let (camera, camera_transform) = *camera;

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };

    let Some(distance) =
        ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
    else {
        return;
    };
    let point = ray.get_point(distance);

    let Ok(prev_ray) = camera.viewport_to_world(camera_transform, cursor_pos - drag.delta) else {
        return;
    };

    let Some(prev_distance) =
        prev_ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
    else {
        return;
    };
    let prev_point = prev_ray.get_point(prev_distance);

    let world_delta = point - prev_point;

    let mut transform = transforms.get_mut(drag.target).unwrap();
    transform.translation.x += world_delta.x;
    transform.translation.z += world_delta.z;
}
