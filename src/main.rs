mod board;
mod pieces;

use bevy::prelude::*;
use board::Board;

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
    .add_systems(PostStartup, add_pieces)
    .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn(Camera2d);
  commands.spawn(Sprite::from_image(asset_server.load("board.png")));
  
  commands.insert_resource(Board::default());
}

fn add_pieces(mut commands: Commands, asset_server: Res<AssetServer>, board: Res<Board>) {
  let half_square_length = 48.0;
  let square_length = 96.0;
  for i in 0..64 {
    let offset_x = (((i % 8) - 4) as f32) * square_length + half_square_length;
    let offset_y = ((4 - (i / 8)) as f32) * square_length - half_square_length;
    let transform = Transform {
      translation: Vec3::new(offset_x, offset_y, 0.1),
      scale: Vec3::splat(0.75),
      rotation: Quat::IDENTITY,
    };

    if (board.white.pawns >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-pawn.png")),
        transform,
      ));
    } else if (board.white.knights >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-knight.png")),
        transform,
      ));
    } else if (board.white.bishops >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-bishop.png")),
        transform,
      ));
    } else if (board.white.rooks >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-rook.png")),
        transform,
      ));
    } else if (board.white.queens >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-queen.png")),
        transform,
      ));
    } else if (board.white.king >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-king.png")),
        transform,
      ));
    } else if (board.black.pawns >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-pawn.png")),
        transform,
      ));
    } else if (board.black.knights >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-knight.png")),
        transform,
      ));
    } else if (board.black.bishops >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-bishop.png")),
        transform,
      ));
    } else if (board.black.rooks >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-rook.png")),
        transform,
      ));
    } else if (board.black.queens >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-queen.png")),
        transform,
      ));
    } else if (board.black.king >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-king.png")),
        transform,
      ));
    }
  }
}
