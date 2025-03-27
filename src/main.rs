mod board;
mod pieces;

use bevy::{prelude::*, window::PrimaryWindow};
use board::Board;

#[derive(Component)]
struct PieceTag;

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
    .add_systems(Update, detect_piece)
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
        PieceTag,
      ));
    } else if (board.white.knights >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-knight.png")),
        transform,
        PieceTag,
      ));
    } else if (board.white.bishops >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-bishop.png")),
        transform,
        PieceTag,
      ));
    } else if (board.white.rooks >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-rook.png")),
        transform,
        PieceTag,
      ));
    } else if (board.white.queens >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-queen.png")),
        transform,
        PieceTag,
      ));
    } else if (board.white.king >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-king.png")),
        transform,
        PieceTag,
      ));
    } else if (board.black.pawns >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-pawn.png")),
        transform,
        PieceTag,
      ));
    } else if (board.black.knights >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-knight.png")),
        transform,
        PieceTag,
      ));
    } else if (board.black.bishops >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-bishop.png")),
        transform,
        PieceTag,
      ));
    } else if (board.black.rooks >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-rook.png")),
        transform,
        PieceTag,
      ));
    } else if (board.black.queens >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-queen.png")),
        transform,
        PieceTag,
      ));
    } else if (board.black.king >> i) & 1 == 1 {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-king.png")),
        transform,
        PieceTag,
      ));
    }
  }
}

fn detect_piece(
  window: Query<&Window, With<PrimaryWindow>>,
  pieces: Query<(&Sprite, &Transform), With<PieceTag>>,
  images: Res<Assets<Image>>,
) {
  if let Some(cursor_position) = &window.single().cursor_position() {
    let resolution = &window.single().resolution;

    //goes from 0 to width / height
    let x = cursor_position.x;
    let y = cursor_position.y;

    for (sprite, transform) in &pieces {
      if let Some(image) = &images.get(sprite.image.id()) {
        let size = image.size();
        let scale = transform.scale.truncate();
        
        let true_size = Vec2 {
          x: size.x as f32 * scale.x,
          y: size.y as f32 * scale.y,
        };

        //transforms are from the pov of a 2d camera so from (-0.5 to 0.5) * width / height
        let image_center = transform.translation.truncate();
        let x0 = image_center.x - true_size.x / 2.0 + resolution.width() / 2.0;
        let x1 = image_center.x + true_size.x / 2.0 + resolution.width() / 2.0;

        let y0 = resolution.height() / 2.0 - image_center.y - true_size.y / 2.0;
        let y1 = resolution.height() / 2.0 - image_center.y + true_size.y / 2.0;

        if x > x0 && x < x1 && y > y0 && y < y1 {
          println!("detected {}", sprite.image.path().unwrap());
        }
      }
    }
  }
}
