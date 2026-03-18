mod font;
mod plugins;
mod state;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use font::setup_font;
use state::GameState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(800, 800),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_systems(Startup, setup_font)
        .add_plugins(plugins::title::TitlePlugin)
        .add_plugins(plugins::game::GamePlugin)
        .add_plugins(plugins::gameover::GameOverPlugin)
        .add_plugins(plugins::gameclear::GameClearPlugin)
        .run();
}
