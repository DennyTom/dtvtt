use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::token;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);
}

fn setup(mut commands: Commands, mut window: Query<Entity, With<Window>>) {
    if let Ok(entity) = window.single_mut() {
        if let Ok(mut window) = commands.get_entity(entity) {
            window
                .observe(token::select_on_click)
                .observe(token::spawn_token_on_click);
        }
    }

    commands.spawn((
        PointLight {
            color: Color::srgb_u8(255, 255, 192),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    commands.spawn((
        PanOrbitCamera {
            button_orbit: MouseButton::Middle,
            button_pan: MouseButton::Right,
            ..default()
        },
        Transform::from_xyz(0.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
