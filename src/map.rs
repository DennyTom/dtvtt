use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Ground;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ground_shape = meshes.add(Plane3d::default().mesh().size(20.0, 20.0));
    let ground_material = materials.add(StandardMaterial::from_color(Color::WHITE));

    commands.spawn((
        Mesh3d(ground_shape.clone()),
        MeshMaterial3d(ground_material.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Ground,
        Pickable::default(),
    ));
}
