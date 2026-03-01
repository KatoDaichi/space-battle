use ::bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_plugins(title::TitlePlugin)
        .add_plugins(game::GamePlugin)
        .add_plugins(gameover::GameOverPlugin)
        .run();
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    /// タイトル画面
    Title,
    /// ゲーム画面
    Game,
    /// ゲームオーバー画面
    GameOver,
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
    use rand::RngExt;

    /// ゲームプレイのプラグイン
    pub struct GamePlugin;

    impl Plugin for GamePlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(
                OnEnter(GameState::Game),
                (setup_camera, setup_ui, setup_player),
            );
            app.init_resource::<EnemySpawnTimer>();
            app.init_resource::<Score>();
            app.add_systems(
                Update,
                (
                    player_movement,
                    shoot_bullet,
                    bullet_movement,
                    enemy_spawner,
                    enemy_movement,
                    check_collisions,
                    update_score_ui,
                )
                    .run_if(in_state(GameState::Game)),
            );
        }
    }

    /// カメラのセットアップ
    fn setup_camera(mut commands: Commands) {
        commands.spawn((Camera2d, DespawnOnExit(GameState::Game)));
    }

    /// スコアのUI用マーカーコンポーネント
    #[derive(Component)]
    struct ScoreText;

    /// スコアを保持するリソース
    #[derive(Resource, Default)]
    struct Score(u32);

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
                    ScoreText,
                ));
            });
    }

    /// スコアのUI表示を更新するシステム
    fn update_score_ui(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
        if score.is_changed() {
            if let Ok(mut text) = query.single_mut() {
                **text = format!("SCORE: {}", score.0);
            }
        }
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
        window_query: Query<&Window>,
        mut query: Query<(&mut Transform, &Sprite), With<Player>>,
    ) {
        // プレイヤーのtransformとspriteを取得
        let Ok((mut transform, sprite)) = query.single_mut() else {
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

        // プレイヤーが画面外に出ないようにクランプ（範囲制限）する
        // スプライトのcustom_sizeの半分を考慮して端にぴったり止まるようにする
        if let Ok(window) = window_query.single() {
            // スプライトの半分のサイズを計算
            let sprite_half = sprite.custom_size.unwrap_or(Vec2::ZERO) / 2.0;
            // ウィンドウの半分のサイズを計算
            let half_w = window.width() / 2.0 - sprite_half.x;
            let half_h = window.height() / 2.0 - sprite_half.y;
            // プレイヤーの位置を指定範囲内に制限する
            transform.translation.x = transform.translation.x.clamp(-half_w, half_w);
            transform.translation.y = transform.translation.y.clamp(-half_h, half_h);
        }
    }

    /// 弾のマーカーコンポーネント
    #[derive(Component)]
    pub struct Bullet;

    /// 弾のサイズ
    const BULLET_SIZE: Vec2 = Vec2::new(10.0, 20.0);

    /// 弾の移動速度（ピクセル/秒）
    const BULLET_SPEED: f32 = 600.0;

    /// Enterキーで弾を発射する処理
    fn shoot_bullet(
        mut commands: Commands,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        query: Query<&Transform, With<Player>>,
    ) {
        // Enterキーが押された時だけ発射する
        if !keyboard_input.just_pressed(KeyCode::Enter) {
            return;
        }

        // プレイヤーの位置を取得
        let Ok(player_transform) = query.single() else {
            return;
        };

        // プレイヤーの位置から弾をspawnする
        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 1.0, 0.0), BULLET_SIZE),
            Transform::from_translation(player_transform.translation),
            Bullet,
            DespawnOnExit(GameState::Game),
        ));
    }

    /// 弾を上方向に移動させる処理
    fn bullet_movement(
        mut commands: Commands,
        time: Res<Time>,
        window_query: Query<&Window>,
        mut query: Query<(Entity, &mut Transform), With<Bullet>>,
    ) {
        // ウィンドウの高さの半分を画面上端のY座標として算出
        // （Bevyの2D座標はY=0が画面中央のため）
        let window_half_height = window_query
            .single()
            .map(|w| w.height() / 2.0)
            .unwrap_or(400.0); // ほぼあり得ないが、ウィンドウの高さを取得できない場合は400.0を代入

        for (entity, mut transform) in &mut query {
            // 弾を上方向に移動
            transform.translation.y += BULLET_SPEED * time.delta_secs();

            // 画面外（上端）に出たら削除する
            if transform.translation.y > window_half_height {
                commands.entity(entity).despawn();
            }
        }
    }

    /// 敵のマーカーコンポーネント
    #[derive(Component)]
    pub struct Enemy;

    /// 敵の移動速度（ピクセル/秒）
    const ENEMY_SPEED: f32 = 200.0;

    /// 敵のスプライトサイズ（正方形）
    const ENEMY_SIZE: Vec2 = Vec2::splat(50.0);

    /// 敵のスポーン間隔を管理するタイマーリソース
    #[derive(Resource)]
    struct EnemySpawnTimer(Timer);

    impl Default for EnemySpawnTimer {
        fn default() -> Self {
            // 2秒ごとに1体スポーンする
            Self(Timer::from_seconds(2.0, TimerMode::Repeating))
        }
    }

    /// 一定間隔でランダムなX座標に敵をspawnする処理
    fn enemy_spawner(
        mut commands: Commands,
        time: Res<Time>,
        mut timer: ResMut<EnemySpawnTimer>,
        window_query: Query<&Window>,
    ) {
        // タイマーを進めて、まだ発火していなければ何もしない
        if !timer.0.tick(time.delta()).just_finished() {
            return;
        }

        // ウィンドウのサイズを取得（取得できなければ何もしない）
        let Ok(window) = window_query.single() else {
            return;
        };

        // rand クレートを使ってランダムなX座標（画面幅の範囲内）を生成する
        // ENEMY_SIZEの半分を差し引いて、敵が画面端からはみ出さないようにする
        let half_w = window.width() / 2.0 - ENEMY_SIZE.x / 2.0;
        let random_x = rand::rng().random_range(-half_w..=half_w);

        // 画面上端のY座標を計算（スプライトの分だけ上にオフセット）
        let spawn_y = window.height() / 2.0 + ENEMY_SIZE.y / 2.0;

        // 敵をspawnする（赤い四角形）
        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 0.2, 0.2), ENEMY_SIZE),
            Transform::from_xyz(random_x, spawn_y, 0.0),
            Enemy,
            DespawnOnExit(GameState::Game),
        ));
    }

    /// 敵を下方向に移動させ、画面外に出たら削除する処理
    fn enemy_movement(
        mut commands: Commands,
        time: Res<Time>,
        window_query: Query<&Window>,
        mut query: Query<(Entity, &mut Transform), With<Enemy>>,
        mut next_state: ResMut<NextState<GameState>>,
    ) {
        // 画面下端のY座標を取得
        let window_half_height = window_query
            .single()
            .map(|w| -(w.height() / 2.0))
            .unwrap_or(-400.0);

        for (entity, mut transform) in &mut query {
            // 敵を下方向に移動
            transform.translation.y -= ENEMY_SPEED * time.delta_secs();

            // 画面外（下端）に出たら削除し、ゲームオーバーにする
            if transform.translation.y < window_half_height - ENEMY_SIZE.y / 2.0 {
                commands.entity(entity).despawn();
                next_state.set(GameState::GameOver);
            }
        }
    }

    /// 弾と敵の当たり判定処理
    fn check_collisions(
        mut commands: Commands,
        bullet_query: Query<(Entity, &Transform, &Sprite), With<Bullet>>,
        enemy_query: Query<(Entity, &Transform, &Sprite), With<Enemy>>,
        mut score: ResMut<Score>,
    ) {
        for (bullet_entity, bullet_transform, bullet_sprite) in &bullet_query {
            // 弾のサイズ
            let bullet_size = bullet_sprite.custom_size.unwrap_or(BULLET_SIZE);
            // 弾の位置
            let b_pos = bullet_transform.translation;

            for (enemy_entity, enemy_transform, enemy_sprite) in &enemy_query {
                // 敵のサイズ
                let enemy_size = enemy_sprite.custom_size.unwrap_or(ENEMY_SIZE);
                // 敵の位置
                let e_pos = enemy_transform.translation;

                // 弾の上下左右の座標
                let b_left = b_pos.x - bullet_size.x / 2.0;
                let b_right = b_pos.x + bullet_size.x / 2.0;
                let b_bottom = b_pos.y - bullet_size.y / 2.0;
                let b_top = b_pos.y + bullet_size.y / 2.0;

                // 敵の上下左右の座標
                let e_left = e_pos.x - enemy_size.x / 2.0;
                let e_right = e_pos.x + enemy_size.x / 2.0;
                let e_bottom = e_pos.y - enemy_size.y / 2.0;
                let e_top = e_pos.y + enemy_size.y / 2.0;

                // シンプルな矩形（AABB）による当たり判定
                let collision =
                    b_left < e_right && b_right > e_left && b_bottom < e_top && b_top > e_bottom;

                if collision {
                    // 当たったら両者を削除する
                    commands.entity(bullet_entity).despawn();
                    commands.entity(enemy_entity).despawn();

                    // スコアを加算
                    score.0 += 1;

                    // この弾は削除予約されたので、他へは当たらないとして次の弾の処理へ移行
                    break;
                }
            }
        }
    }
}

/// ゲームオーバー画面
mod gameover {
    use super::*;

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
                DespawnOnExit(GameState::GameOver),
            ))
            .with_children(|parent| {
                // ゲームオーバーテキスト
                parent.spawn((
                    Text::new("GAME OVER"),
                    TextFont {
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
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
    }
}
