use bevy::prelude::*;
use bevy_mod_outline::OutlinePlugin;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

mod main_scene;
mod map;
mod token;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Window {
                    title: "DTVTT".to_string(),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }
                .into(),
                ..default()
            }),
            MeshPickingPlugin,
            OutlinePlugin,
            PanOrbitCameraPlugin,
            token::plugin,
            map::plugin,
            main_scene::plugin,
        ))
        .run();
}
