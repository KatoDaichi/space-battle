use bevy::prelude::*;

use crate::font::DefaultFont;
use crate::state::GameState;

/// ゲームオーバー画面のプラグイン
pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), (setup_camera, setup_ui));
        app.add_systems(
            Update,
            gameover_update.run_if(in_state(GameState::GameOver)),
        );
    }
}

/// ゲームオーバー画面の更新処理（Rキーでリトライ、Enterでタイトル）
fn gameover_update(
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
    commands.spawn((Camera2d, DespawnOnExit(GameState::GameOver)));
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
            DespawnOnExit(GameState::GameOver),
        ))
        .with_children(|parent| {
            // ゲームオーバーテキスト
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font: asset.font.clone(),
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
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
