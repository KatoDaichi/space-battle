use ::bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_plugins(title::TitlePlugin)
        .add_plugins(game::GamePlugin)
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

/// ゲーム画面
mod game {
    use super::*;

    /// ゲームプレイのプラグイン
    pub struct GamePlugin;

    impl Plugin for GamePlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(
                OnEnter(GameState::Game),
                (setup_camera, setup_ui, setup_player),
            );
            app.add_systems(Update, player_movement.run_if(in_state(GameState::Game)));
        }
    }

    /// カメラのセットアップ
    fn setup_camera(mut commands: Commands) {
        commands.spawn((Camera2d, DespawnOnExit(GameState::Game)));
    }

    /// ゲーム画面のUIセットアップ
    fn setup_ui(mut commands: Commands) {
        commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                DespawnOnExit(GameState::Game),
            ))
            .with_children(|parent| {
                // スコア表示等
                parent.spawn((
                    Text::new("SCORE: 0"),
                    TextFont {
                        font_size: 30.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
    }

    /// プレイヤーのマーカーコンポーネント
    #[derive(Component)]
    pub struct Player;

    /// プレイヤーのセットアップ
    fn setup_player(mut commands: Commands) {
        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 1.0, 1.0), Vec2::new(50.0, 50.0)),
            Transform::from_xyz(0.0, -250.0, 0.0),
            Player,
            DespawnOnExit(GameState::Game),
        ));
    }

    /// プレイヤーの移動速度（ピクセル/秒）
    const PLAYER_SPEED: f32 = 300.0;

    /// プレイヤーの移動処理
    fn player_movement(
        keyboard_input: Res<ButtonInput<KeyCode>>,
        time: Res<Time>,
        mut query: Query<&mut Transform, With<Player>>,
    ) {
        // プレイヤーのtransformを取得
        let Ok(mut transform) = query.single_mut() else {
            return;
        };

        // どの方向に向かって進かの情報を初期化
        let mut direction = Vec2::ZERO;

        // Wを押すと上方向に進む
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        // Sを押すと下方向に進む
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        // Aを押すと左方向に進む
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        // Dを押すと右方向に進む
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        // 斜め移動などで方向ベクトルの長さが1.0を超えた場合、
        // 斜め移動の時に、縦横移動より多くの距離を進むことを防ぐため
        // ベクトルの長さ（大きさ）がちょうど 1.0 になるように正規化する
        // （結果として、xとyはそれぞれ -1.0 ~ 1.0 の間の値になる）
        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        // 移動先のx座標とy座標を設定
        // direction（-1.0 ~ 1.0） * プレイヤーの移動速度 * 前フレームからの経過時間
        // を掛けることで、「1秒間に約300ピクセル進む」一定の速度になる
        transform.translation.x += direction.x * PLAYER_SPEED * time.delta_secs();
        transform.translation.y += direction.y * PLAYER_SPEED * time.delta_secs();
    }
}
