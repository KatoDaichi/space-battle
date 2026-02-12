use ::bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_plugins(title::TitlePlugin)
        .run();
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    /// タイトル画面
    Title,
    /// ゲーム画面
    Game,
}

/// タイトル画面
mod title {
    use super::*;

    /// タイトル画面のプラグイン
    pub struct TitlePlugin;

    impl Plugin for TitlePlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(OnEnter(GameState::Title), (setup_camera, setup_ui));
        }
    }

    // カメラのセットアップ
    fn setup_camera(mut commands: Commands) {
        commands.spawn((Camera2d, DespawnOnExit(GameState::Title)));
    }

    // UIのセットアップ
    fn setup_ui(mut commands: Commands) {
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
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
    }
}
