use bevy::prelude::*;

/// ゲームの状態を表す列挙型
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
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
