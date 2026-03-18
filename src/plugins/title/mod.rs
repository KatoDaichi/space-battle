use bevy::prelude::*;
use crate::font::DefaultFont;
use crate::font::setup_font;
use crate::state::GameState;
use crate::DespawnOnExit;

/// タイトル画面のプラグイン
pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Title),
            (setup_camera, (setup_font, setup_ui).chain()),
        );
        app.add_systems(Update, title_update.run_if(in_state(GameState::Title)));
    }
}

/// タイトル画面の更新処理（Enterキーで遷移）
fn title_update(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Game);
    }
}

/// カメラのセットアップ
fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, DespawnOnExit(GameState::Title)));
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
            DespawnOnExit(GameState::Title),
        ))
        .with_children(|parent| {
            // タイトルテキスト
            parent.spawn((
                Text::new("SPACE BATTLE"),
                TextFont {
                    font: asset.font.clone(),
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // 説明テキスト
            parent.spawn((
                Text::new("Press Enter to Start"),
                TextFont {
                    font: asset.font.clone(),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}
