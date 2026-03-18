use bevy::prelude::*;

/// フォントを保持するリソース
#[derive(Resource)]
pub struct DefaultFont {
    pub font: Handle<Font>,
}

/// フォントのセットアップ
pub fn setup_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle: Handle<Font> = asset_server.load("fonts/DotGothic16-Regular.ttf");
    commands.insert_resource(DefaultFont { font: font_handle });
}
