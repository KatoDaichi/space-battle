use bevy::prelude::*;
use bevy::window::WindowResolution;

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
        .add_plugins(title::TitlePlugin)
        .add_plugins(game::GamePlugin)
        .add_plugins(gameover::GameOverPlugin)
        .add_plugins(gameclear::GameClearPlugin)
        .run();
}

/// フォントを保持するリソース
#[derive(Resource)]
struct DefaultFont {
    font: Handle<Font>,
}

/// フォントのセットアップ
fn setup_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle: Handle<Font> = asset_server.load("fonts/DotGothic16-Regular.ttf");
    commands.insert_resource(DefaultFont { font: font_handle });
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
    /// ゲームクリア画面
    GameClear,
}

/// タイトル画面
mod title {
    use super::*;

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
                (setup_camera, setup_ui, setup_player, reset_game_elapsed),
            );
            app.init_resource::<EnemySpawnTimer>();
            app.init_resource::<EnemyCount>();
            app.init_resource::<GameElapsedTime>();
            app.add_sub_state::<PauseState>();
            app.add_systems(OnEnter(PauseState::Paused), setup_pause_ui);
            app.add_systems(OnExit(PauseState::Paused), despawn_pause_ui);
            app.add_systems(Update, pause_update.run_if(in_state(PauseState::Paused)));
            app.add_systems(Update, toggle_pause.run_if(in_state(PauseState::Running)));
            app.add_systems(
                Update,
                (
                    tick_game_elapsed,
                    player_movement,
                    charge_bullets,
                    shoot_bullet,
                    bullet_movement,
                    enemy_spawner,
                    enemy_movement,
                    check_player_enemy_collision,
                    check_bullet_enemy_collisions,
                    update_enemy_count_ui,
                    update_hp_ui,
                    update_bullet_ui,
                )
                    .run_if(in_state(PauseState::Running)),
            );
        }
    }

    /// ポーズ状態（GameState::Gameのサブステート）
    #[derive(SubStates, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
    #[source(GameState = GameState::Game)]
    enum PauseState {
        #[default]
        Running,
        Paused,
    }

    /// ポーズUI用マーカーコンポーネント
    #[derive(Component)]
    struct PauseScreen;

    /// ポーズオーバーレイUIを生成する
    fn setup_pause_ui(mut commands: Commands, asset: Res<DefaultFont>) {
        commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ZIndex(100),
                PauseScreen,
            ))
            .with_children(|parent| {
                // PAUSED テキスト
                parent.spawn((
                    Text::new("PAUSED"),
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

                // 操作説明
                parent.spawn((
                    Text::new("Escapeで続行"),
                    TextFont {
                        font: asset.font.clone(),
                        font_size: 36.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::bottom(Val::Px(16.0)),
                        ..default()
                    },
                ));

                parent.spawn((
                    Text::new("Enterでタイトルへ"),
                    TextFont {
                        font: asset.font.clone(),
                        font_size: 36.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                ));
            });
    }

    /// ポーズUIを削除する
    fn despawn_pause_ui(mut commands: Commands, query: Query<Entity, With<PauseScreen>>) {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }

    /// Escapeキーでポーズを開始する（Running中のみ）
    fn toggle_pause(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut next_pause: ResMut<NextState<PauseState>>,
    ) {
        if keyboard.just_pressed(KeyCode::Escape) {
            next_pause.set(PauseState::Paused);
        }
    }

    /// ポーズ中の操作処理
    fn pause_update(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut next_pause: ResMut<NextState<PauseState>>,
        mut next_game: ResMut<NextState<GameState>>,
    ) {
        if keyboard.just_pressed(KeyCode::Escape) {
            // Escapeで続行
            next_pause.set(PauseState::Running);
        } else if keyboard.just_pressed(KeyCode::Enter) {
            // Enterでタイトルへ戻る
            next_game.set(GameState::Title);
        }
    }

    /// カメラのセットアップ
    fn setup_camera(mut commands: Commands) {
        commands.spawn((Camera2d, DespawnOnExit(GameState::Game)));
    }

    /// スコアのUI用マーカーコンポーネント
    #[derive(Component)]
    struct ScoreText;

    /// HP アイコン行のマーカーコンポーネント
    #[derive(Component)]
    struct HpIcons;

    /// 残弾アイコン行のマーカーコンポーネント
    #[derive(Component)]
    struct BulletIcons;

    /// 残り討伐数を保持するリソース（100体からカウントダウン）
    const ENEMY_TOTAL: u32 = 100;

    #[derive(Resource)]
    struct EnemyCount(u32);

    impl Default for EnemyCount {
        fn default() -> Self {
            Self(ENEMY_TOTAL)
        }
    }

    /// アイコンのサイズ
    const ICON_SIZE: f32 = 20.0;
    /// アイコン間のマージン
    const ICON_MARGIN: f32 = 4.0;

    /// ゲーム画面のUIセットアップ
    fn setup_ui(mut commands: Commands, asset: Res<DefaultFont>) {
        commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                DespawnOnExit(GameState::Game),
            ))
            .with_children(|parent| {
                // 残り敵数表示
                parent.spawn((
                    Text::new(format!("ENEMY: {}", ENEMY_TOTAL)),
                    TextFont {
                        font: asset.font.clone(),
                        font_size: 30.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::bottom(Val::Px(6.0)),
                        ..default()
                    },
                    ScoreText,
                ));

                // HP アイコン行
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                    HpIcons,
                ));

                // 残弾アイコン行
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BulletIcons,
                ));
            });
    }

    /// 残り敵数のUI表示を更新するシステム
    fn update_enemy_count_ui(
        enemy_count: Res<EnemyCount>,
        mut query: Query<&mut Text, With<ScoreText>>,
    ) {
        if enemy_count.is_changed() {
            if let Ok(mut text) = query.single_mut() {
                **text = format!("ENEMY: {}", enemy_count.0);
            }
        }
    }

    /// HP アイコンを再描画するシステム
    fn update_hp_ui(
        mut commands: Commands,
        player_query: Query<&HP, (With<Player>, Changed<HP>)>,
        icons_query: Query<Entity, With<HpIcons>>,
    ) {
        let Ok(hp) = player_query.single() else {
            return;
        };
        let Ok(container) = icons_query.single() else {
            return;
        };

        // 既存の子エンティティをすべて削除して再描画
        commands.entity(container).despawn_related::<Children>();

        commands.entity(container).with_children(|parent| {
            for _ in 0..hp.0 {
                parent.spawn((
                    Node {
                        width: Val::Px(ICON_SIZE),
                        height: Val::Px(ICON_SIZE),
                        margin: UiRect::right(Val::Px(ICON_MARGIN)),
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                ));
            }
        });
    }

    /// 残弾アイコンを再描画するシステム
    fn update_bullet_ui(
        mut commands: Commands,
        player_query: Query<&BulletStock, (With<Player>, Changed<BulletStock>)>,
        icons_query: Query<Entity, With<BulletIcons>>,
    ) {
        let Ok(stock) = player_query.single() else {
            return;
        };
        let Ok(container) = icons_query.single() else {
            return;
        };

        // 既存の子エンティティをすべて削除して再描画
        commands.entity(container).despawn_related::<Children>();

        commands.entity(container).with_children(|parent| {
            for _ in 0..stock.current {
                parent.spawn((
                    Node {
                        width: Val::Px(ICON_SIZE * 0.6),
                        height: Val::Px(ICON_SIZE),
                        margin: UiRect::right(Val::Px(ICON_MARGIN)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(1.0, 0.85, 0.0)),
                ));
            }
        });
    }

    /// プレイヤーのマーカーコンポーネント
    #[derive(Component)]
    pub struct Player;

    /// HPコンポーネント
    #[derive(Component)]
    pub struct HP(u32);

    /// プレイヤーの移動速度（ピクセル/秒）
    const PLAYER_SPEED: f32 = 350.0;
    /// プレイヤーのサイズ
    const PLAYER_SIZE: Vec2 = Vec2::new(50.0, 50.0);
    /// プレイヤーのHP
    const PLAYER_HP: u32 = 3;
    /// 弾の最大ストック数
    const MAX_BULLET_STOCK: u32 = 3;
    /// 弾が1発チャージされるまでの秒数
    const BULLET_CHARGE_SECS: f32 = 1.0;

    /// 弾のストックを管理するコンポーネント
    #[derive(Component)]
    struct BulletStock {
        /// 現在の残弾数
        current: u32,
        /// 次のチャージまでの経過時間（秒）
        charge_timer: f32,
    }

    impl Default for BulletStock {
        fn default() -> Self {
            Self {
                current: MAX_BULLET_STOCK,
                charge_timer: 0.0,
            }
        }
    }

    /// プレイヤーのセットアップ
    fn setup_player(mut commands: Commands) {
        commands.spawn((
            Sprite::from_color(Color::WHITE, PLAYER_SIZE),
            Transform::from_xyz(0.0, -250.0, 0.0),
            Player,
            HP(PLAYER_HP),
            BulletStock::default(),
            DespawnOnExit(GameState::Game),
        ));
    }

    /// 時間経過で弾をチャージするシステム
    fn charge_bullets(time: Res<Time>, mut query: Query<&mut BulletStock, With<Player>>) {
        let Ok(mut stock) = query.single_mut() else {
            return;
        };

        // すでに最大ストックなら何もしない
        if stock.current >= MAX_BULLET_STOCK {
            stock.charge_timer = 0.0;
            return;
        }

        // 経過時間を加算
        stock.charge_timer += time.delta_secs();
        // チャージ時間を経過した場合
        if stock.charge_timer >= BULLET_CHARGE_SECS {
            // 経過時間をリセット
            stock.charge_timer -= BULLET_CHARGE_SECS;
            // 弾をチャージ
            stock.current += 1;
        }
    }

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
        mut query: Query<(&Transform, &mut BulletStock), With<Player>>,
    ) {
        // Enterキーが押された時だけ発射する
        if !keyboard_input.just_pressed(KeyCode::Enter) {
            return;
        }

        // プレイヤーの位置と弾ストックを取得
        let Ok((player_transform, mut stock)) = query.single_mut() else {
            return;
        };

        // 残弾がなければ発射しない
        if stock.current == 0 {
            return;
        }

        // 残弾を1消費して弾をspawnする
        stock.current -= 1;

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

    /// 敵のスプライトサイズの初期値（正方形）
    const ENEMY_SIZE_INITIAL: Vec2 = Vec2::splat(50.0);
    /// 敵のスプライトサイズの最小値
    const ENEMY_SIZE_MIN: f32 = 25.0;
    /// 何秒ごとにサイズを縮小するか
    const ENEMY_SIZE_STEP_SECS: f32 = 20.0;
    /// 1ステップあたりの縮小量
    const ENEMY_SIZE_STEP_AMOUNT: f32 = 5.0;

    /// 敵のスポーン間隔を管理するタイマーリソース
    #[derive(Resource)]
    struct EnemySpawnTimer(Timer);

    impl Default for EnemySpawnTimer {
        fn default() -> Self {
            // 2秒ごとに1体スポーンする
            Self(Timer::from_seconds(2.0, TimerMode::Repeating))
        }
    }

    /// スポーン間隔の初期値（秒）
    const SPAWN_INTERVAL_INITIAL: f32 = 2.0;
    /// スポーン間隔の最小値（秒）
    const SPAWN_INTERVAL_MIN: f32 = 1.0;
    /// 何秒ごとに間隔を短縮するか
    const SPAWN_INTERVAL_STEP_SECS: f32 = 10.0;
    /// 1ステップあたりの短縮量（秒）
    const SPAWN_INTERVAL_STEP_AMOUNT: f32 = 0.1;

    /// ゲーム開始からの経過時間（秒）を管理するリソース
    #[derive(Resource, Default)]
    struct GameElapsedTime(f32);

    /// 毎フレーム経過時間を加算するシステム
    fn tick_game_elapsed(time: Res<Time>, mut game_elapsed_time: ResMut<GameElapsedTime>) {
        game_elapsed_time.0 += time.delta_secs();
    }

    /// ゲーム開始時に経過時間・残り敵数をリセットするシステム
    fn reset_game_elapsed(
        mut game_elapsed_time: ResMut<GameElapsedTime>,
        mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
        mut enemy_count: ResMut<EnemyCount>,
    ) {
        // ゲーム内経過時間を0.0秒にリセット
        game_elapsed_time.0 = 0.0;
        // 敵のスポーン間隔タイマーも初期間隔に戻す
        enemy_spawn_timer.0 = Timer::from_seconds(SPAWN_INTERVAL_INITIAL, TimerMode::Repeating);
        // 残り敵数を初期値に戻す
        enemy_count.0 = ENEMY_TOTAL;
    }

    /// 一定間隔でランダムなX座標に敵をspawnする処理
    fn enemy_spawner(
        mut commands: Commands,
        time: Res<Time>,
        mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
        game_elapsed_time: Res<GameElapsedTime>,
        window_query: Query<&Window>,
    ) {
        // 敵のスポーン間隔タイマーを進めて、まだ発火していなければ何もしない
        if !enemy_spawn_timer.0.tick(time.delta()).just_finished() {
            return;
        }

        // ウィンドウのサイズを取得（取得できなければ何もしない）
        let Ok(window) = window_query.single() else {
            return;
        };

        // 経過時間に応じて敵のサイズを計算する
        let size_steps = (game_elapsed_time.0 / ENEMY_SIZE_STEP_SECS).floor();
        let enemy_side =
            (ENEMY_SIZE_INITIAL.x - size_steps * ENEMY_SIZE_STEP_AMOUNT).max(ENEMY_SIZE_MIN);
        let enemy_size = Vec2::splat(enemy_side);

        // rand クレートを使ってランダムなX座標（画面幅の範囲内）を生成する
        // enemy_sizeの半分を差し引いて、敵が画面端からはみ出さないようにする
        let half_w = window.width() / 2.0 - enemy_size.x / 2.0;
        let random_x = rand::rng().random_range(-half_w..=half_w);

        // 画面上端のY座標を計算（スプライトの分だけ上にオフセット）
        let spawn_y = window.height() / 2.0 + enemy_size.y / 2.0;

        // 敵をspawnする（赤い四角形）
        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 0.2, 0.2), enemy_size),
            Transform::from_xyz(random_x, spawn_y, 0.0),
            Enemy,
            DespawnOnExit(GameState::Game),
        ));

        // ゲーム内経過時間に応じて敵スポーンタイマーの間隔を更新する
        let interval_steps = (game_elapsed_time.0 / SPAWN_INTERVAL_STEP_SECS).floor();
        let new_interval = (SPAWN_INTERVAL_INITIAL - interval_steps * SPAWN_INTERVAL_STEP_AMOUNT)
            .max(SPAWN_INTERVAL_MIN);
        enemy_spawn_timer
            .0
            .set_duration(std::time::Duration::from_secs_f32(new_interval));
    }

    /// 敵を下方向に移動させ、画面外に出たらHPを減らす処理
    fn enemy_movement(
        mut commands: Commands,
        time: Res<Time>,
        window_query: Query<&Window>,
        mut query: Query<(Entity, &mut Transform), With<Enemy>>,
        mut player_query: Query<(Entity, &mut HP), With<Player>>,
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

            // 画面外（下端）に出たら削除し、プレイヤーのHPを1減らす
            if transform.translation.y < window_half_height - ENEMY_SIZE_INITIAL.y / 2.0 {
                commands.entity(entity).despawn();

                if let Ok((player_entity, mut hp)) = player_query.single_mut() {
                    if hp.0 > 1 {
                        // HPが残っていれば1減らす
                        hp.0 -= 1;
                    } else {
                        // HPが0になったらゲームオーバー
                        commands.entity(player_entity).despawn();
                        next_state.set(GameState::GameOver);
                    }
                }
            }
        }
    }

    /// プレイヤーと敵の当たり判定処理
    fn check_player_enemy_collision(
        mut commands: Commands,
        mut player_query: Query<(Entity, &Transform, &Sprite, &mut HP), With<Player>>,
        enemy_query: Query<(Entity, &Transform, &Sprite), With<Enemy>>,
        mut next_state: ResMut<NextState<GameState>>,
    ) {
        for (player_entity, player_transform, player_sprite, mut player_hp) in &mut player_query {
            // プレイヤーのサイズ
            let player_size = player_sprite.custom_size.unwrap_or(PLAYER_SIZE);
            // プレイヤーの位置
            let p_pos = player_transform.translation;

            // プレイヤーの上下左右の座標
            let p_left = p_pos.x - player_size.x / 2.0;
            let p_right = p_pos.x + player_size.x / 2.0;
            let p_bottom = p_pos.y - player_size.y / 2.0;
            let p_top = p_pos.y + player_size.y / 2.0;

            for (enemy_entity, enemy_transform, enemy_sprite) in &enemy_query {
                // 敵のサイズ
                let enemy_size = enemy_sprite.custom_size.unwrap_or(ENEMY_SIZE_INITIAL);
                // 敵の位置
                let e_pos = enemy_transform.translation;

                // 敵の上下左右の座標
                let e_left = e_pos.x - enemy_size.x / 2.0;
                let e_right = e_pos.x + enemy_size.x / 2.0;
                let e_bottom = e_pos.y - enemy_size.y / 2.0;
                let e_top = e_pos.y + enemy_size.y / 2.0;

                // シンプルな矩形（AABB）による当たり判定
                let collision =
                    p_left < e_right && p_right > e_left && p_bottom < e_top && p_top > e_bottom;

                if collision {
                    // 当たったら敵を削除する
                    commands.entity(enemy_entity).despawn();
                    // プレイヤーのHPが1以上ならHPを減らす、0ならゲームオーバー
                    if player_hp.0 > 1 {
                        player_hp.0 -= 1;
                    } else {
                        commands.entity(player_entity).despawn();
                        next_state.set(GameState::GameOver);
                    }
                }
            }
        }
    }

    /// 弾と敵の当たり判定処理
    fn check_bullet_enemy_collisions(
        mut commands: Commands,
        bullet_query: Query<(Entity, &Transform, &Sprite), With<Bullet>>,
        enemy_query: Query<(Entity, &Transform, &Sprite), With<Enemy>>,
        mut enemy_count: ResMut<EnemyCount>,
        mut next_state: ResMut<NextState<GameState>>,
    ) {
        for (bullet_entity, bullet_transform, bullet_sprite) in &bullet_query {
            // 弾のサイズ
            let bullet_size = bullet_sprite.custom_size.unwrap_or(BULLET_SIZE);
            // 弾の位置
            let b_pos = bullet_transform.translation;

            for (enemy_entity, enemy_transform, enemy_sprite) in &enemy_query {
                // 敵のサイズ
                let enemy_size = enemy_sprite.custom_size.unwrap_or(ENEMY_SIZE_INITIAL);
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

                    // 残り敵数を1減らす
                    if enemy_count.0 > 0 {
                        enemy_count.0 -= 1;
                    }

                    // 残り敵数が0になったらゲームクリア
                    if enemy_count.0 == 0 {
                        next_state.set(GameState::GameClear);
                    }

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
}

/// ゲームクリア画面
mod gameclear {
    use super::*;

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
}
