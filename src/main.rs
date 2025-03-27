use bevy::prelude::*;

fn main() {
  let plugins = DefaultPlugins.set(WindowPlugin {
    primary_window: Some(Window {
      resolution: (784.0, 784.0).into(),
      resizable: false,
      ..default()
    }),
    ..default()
  });

  App::new()
    .add_plugins(plugins)
    .add_systems(Startup, startup)
    .run();
}

fn startup(
  mut commands: Commands,
  asset_server: Res<AssetServer>
) {
  commands.spawn(Camera2d);

  let _offset = 8_usize;
  let _square_length = 96_usize;
  let _total_length = 784;

  commands.spawn(
    Sprite::from_image(asset_server.load("board.png"))
  );
}
