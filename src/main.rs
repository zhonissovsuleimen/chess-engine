mod board;
mod board_position_lookup;
mod pieces;

use bevy::{prelude::*, window::PrimaryWindow};
use board::Board;
use board_position_lookup::LOOKUP;

#[derive(Component)]
struct PieceTag;

struct SelectedPieceData {
  entity: Entity,
  original_translation: Vec3,
}

#[derive(Resource)]
struct SelectedPiece {
  data: Option<SelectedPieceData>,
}

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
  commands.insert_resource(SelectedPiece { data: None });
}

fn add_pieces(mut commands: Commands, asset_server: Res<AssetServer>, board: Res<Board>) {
  for i in 0..64 {
    let transform = Transform {
      translation: LOOKUP[i],
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
  mut pieces: Query<(Entity, &Sprite, &mut Transform), With<PieceTag>>,
  images: Res<Assets<Image>>,
  mouse: Res<ButtonInput<MouseButton>>,
  mut selected_piece: ResMut<SelectedPiece>,
) {
  let Some(cursor_position) = window.single().cursor_position() else {
    return;
  };
  //goes from 0 to width or height
  let x = cursor_position.x;
  let y = cursor_position.y;
  
  let resolution = &window.single().resolution;
  
  let just_pressed = mouse.just_pressed(MouseButton::Left);
  let being_pressed = mouse.pressed(MouseButton::Left);
  let just_released = mouse.just_released(MouseButton::Left);
  
  match &mut selected_piece.data {
    Some(data) if being_pressed => {
      if let Ok((_, _, mut transform)) = pieces.get_mut(data.entity) {
        transform.translation.x = cursor_position.x - resolution.width() / 2.0;
        transform.translation.y = resolution.height() / 2.0 - cursor_position.y;
        transform.translation.z = 1.0;
      }
    }
    Some(data) if just_released => {
      if let Ok((_, _, mut transform)) = pieces.get_mut(data.entity) {
        transform.translation = data.original_translation;
      }
      selected_piece.data = None;
    }
    None if just_pressed => {
      for (entity, sprite, transform) in &pieces {
        if let Some(image) = &images.get(sprite.image.id()) {
          let size = image.size();
          let scale = transform.scale.truncate();

          let true_size = Vec2 {
            x: size.x as f32 * scale.x,
            y: size.y as f32 * scale.y,
          };

          //transforms are from the pov of a 2d camera so from (-0.5 to 0.5) * width or height
          let image_center = transform.translation.truncate();
          let x0 = image_center.x - true_size.x / 2.0 + resolution.width() / 2.0;
          let x1 = image_center.x + true_size.x / 2.0 + resolution.width() / 2.0;

          let y0 = resolution.height() / 2.0 - image_center.y - true_size.y / 2.0;
          let y1 = resolution.height() / 2.0 - image_center.y + true_size.y / 2.0;

          if x > x0 && x < x1 && y > y0 && y < y1 {
            if mouse.just_pressed(MouseButton::Left) {
              let data = SelectedPieceData {
                entity,
                original_translation: transform.translation,
              };

              selected_piece.data = Some(data);
            }
          }
        }
      }
    }
    _ => {}
  }
}

