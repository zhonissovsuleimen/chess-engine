mod board;
mod board_position_lookup;
mod pieces;

use bevy::{prelude::*, window::PrimaryWindow};
use board::Board;
use board_position_lookup::{CENTER_LOOKUP, X_LOOKUP, Y_LOOKUP};

#[derive(Component)]
struct PieceTag(bool);

struct SelectedPieceData {
  entity: Entity,
  original_translation: Vec3,
  original_board_pos: usize,
}

#[derive(Resource, Default)]
struct SelectedPiece {
  data: Option<SelectedPieceData>,
}

#[derive(Resource, Default)]
struct MouseData {
  x: f32,
  y: f32,
  board_pos: usize,

  being_pressed: bool,
  just_pressed: bool,
  just_released: bool,
}

#[derive(Resource)]
struct BoardValue {
  value: i32
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
    .add_systems(PreStartup, startup)
    .add_systems(PreUpdate, (update_sprites, update_mouse_data))
    .add_systems(Update, detect_piece)
    .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn(Camera2d);
  commands.spawn(Sprite::from_image(asset_server.load("board.png")));

  commands.insert_resource(Board::default());
  commands.insert_resource(SelectedPiece::default());
  commands.insert_resource(MouseData::default());
  commands.insert_resource(BoardValue { value: i32::MAX });
}

fn detect_piece(
  sprites: Res<Assets<Image>>,
  mut sprite_query: Query<(Entity, &Sprite, &mut Transform, &PieceTag)>,
  mouse: Res<MouseData>,
  mut selected_piece: ResMut<SelectedPiece>,
  mut board: ResMut<Board>,
) {
  match &mut selected_piece.data {
    Some(data) if mouse.being_pressed => {
      if let Ok((_, _, mut transform, _)) = sprite_query.get_mut(data.entity) {
        transform.translation.x = mouse.x;
        transform.translation.y = mouse.y;
        transform.translation.z = 1.0;
      }
    }
    Some(data) if mouse.just_released => {
      if let Ok((_, _, mut transform, _)) = sprite_query.get_mut(data.entity) {
        if board.move_piece(data.original_board_pos, mouse.board_pos) {
          transform.translation = CENTER_LOOKUP[mouse.board_pos];
        } else {
          transform.translation = data.original_translation;
        }
      }
      selected_piece.data = None;
    }
    None if mouse.just_pressed => {
      for (entity, sprite, transform, is_white) in &sprite_query {
        if is_white.0 != board.white_to_move {
          continue;
        }

        if let Some(image) = &sprites.get(sprite.image.id()) {
          let size = image.size();
          let scale = transform.scale.truncate();

          let true_size = Vec2 {
            x: size.x as f32 * scale.x,
            y: size.y as f32 * scale.y,
          };

          //transforms are from the pov of a 2d camera so from (-0.5 to 0.5) * width or height
          let image_center = transform.translation.truncate();
          let x0 = image_center.x - true_size.x / 2.0;
          let x1 = image_center.x + true_size.x / 2.0;

          let y0 = image_center.y - true_size.y / 2.0;
          let y1 = image_center.y + true_size.y / 2.0;

          if mouse.x > x0 && mouse.x < x1 && mouse.y > y0 && mouse.y < y1 {
            let data = SelectedPieceData {
              entity,
              original_translation: transform.translation,
              original_board_pos: mouse.board_pos,
            };

            selected_piece.data = Some(data);
          }
        }
      }
    }
    _ => {}
  }
}

fn update_sprites(mut commands: Commands, asset_server: Res<AssetServer>, board: Res<Board>, mut prev_value: ResMut<BoardValue>, sprites: Query<Entity, With<PieceTag>>) {
  if board.get_piece_delta() == prev_value.value {
    return;
  }
  prev_value.value = board.get_piece_delta();
  
  for sprite in sprites.iter() {
    commands.entity(sprite).despawn();
  }

  for i in 0..64 {
    let at_mask = 1 << i;

    let transform = Transform {
      translation: CENTER_LOOKUP[i],
      scale: Vec3::splat(0.75),
      rotation: Quat::IDENTITY,
    };

    if board.white.is_pawn(at_mask) {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-pawn.png")),
        transform,
        PieceTag(true),
      ));
    } else if board.white.is_knight(at_mask) {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-knight.png")),
        transform,
        PieceTag(true),
      ));
    } else if board.white.is_bishop(at_mask) {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-bishop.png")),
        transform,
        PieceTag(true),
      ));
    } else if board.white.is_rook(at_mask) {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-rook.png")),
        transform,
        PieceTag(true),
      ));
    } else if board.white.is_queen(at_mask) {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-queen.png")),
        transform,
        PieceTag(true),
      ));
    } else if board.white.is_king(at_mask) {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/white-king.png")),
        transform,
        PieceTag(true),
      ));
    } else if board.black.is_pawn(at_mask) {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-pawn.png")),
        transform,
        PieceTag(false),
      ));
    } else if board.black.is_knight(at_mask) {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-knight.png")),
        transform,
        PieceTag(false),
      ));
    } else if board.black.is_bishop(at_mask) {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-bishop.png")),
        transform,
        PieceTag(false),
      ));
    } else if board.black.is_rook(at_mask) {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-rook.png")),
        transform,
        PieceTag(false),
      ));
    } else if board.black.is_queen(at_mask) {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-queen.png")),
        transform,
        PieceTag(false),
      ));
    } else if board.black.is_king(at_mask) {
      commands.spawn((
        Sprite::from_image(asset_server.load("pieces/black-king.png")),
        transform,
        PieceTag(false),
      ));
    }
  }
}

fn update_mouse_data(
  mut data: ResMut<MouseData>,
  window: Query<&Window, With<PrimaryWindow>>,
  mouse: Res<ButtonInput<MouseButton>>,
) {
  data.being_pressed = mouse.pressed(MouseButton::Left);
  data.just_pressed = mouse.just_pressed(MouseButton::Left);
  data.just_released = mouse.just_released(MouseButton::Left);

  let Some(cursor) = &window.single().cursor_position() else {
    return;
  };
  let resolution = &window.single().resolution;

  let x = cursor.x - resolution.width() / 2.0;
  let y = resolution.height() / 2.0 - cursor.y;

  data.x = x;
  data.y = y;

  for i in 0..64 {
    if x > X_LOOKUP[i].0 && x < X_LOOKUP[i].1 && y > Y_LOOKUP[i].0 && y < Y_LOOKUP[i].1 {
      data.board_pos = i;
      return;
    }
  }
}
