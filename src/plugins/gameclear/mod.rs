use bevy::prelude::*;

use crate::font::DefaultFont;
use crate::state::GameState;

/// ゲームクリア画面のプラグイン
pub struct GameClearPlugin;

impl Plugin for GameClearPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameClear), (setup_camera, setup_ui));
        app.add_systems(
            Update,
            gameclear_update.run_if(in_state(GameState::GameClear)),
        );
    }
}

/// ゲームクリア画面の更新処理（Rキーでリトライ、Enterでタイトル）
fn gameclear_update(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Title);
    } else if keyboard_input.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Game);
    }
}

/// カメラのセットアップ
fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, DespawnOnExit(GameState::GameClear)));
}

/// UIのセットアップ
fn setup_ui(mut commands: Commands, asset: Res<DefaultFont>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            DespawnOnExit(GameState::GameClear),
        ))
        .with_children(|parent| {
            // ゲームクリアテキスト
            parent.spawn((
                Text::new("GAME CLEAR!"),
                TextFont {
                    font: asset.font.clone(),
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(0.2, 1.0, 0.4)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // 説明テキスト
            parent.spawn((
                Text::new("Press R to Retry\nPress Enter to Title"),
                TextFont {
                    font: asset.font.clone(),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}
