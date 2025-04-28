mod board;
mod board_assets;
mod board_position_lookup;

use bevy::{prelude::*, window::PrimaryWindow};
use board::{Board, MoveInput};
use board_assets::{BoardAssets, PieceTag};
use board_position_lookup::{X_LOOKUP, Y_LOOKUP};

#[derive(Resource)]
struct SelectedPiece {
  data: Option<(Entity, usize)>,
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
    .add_systems(Startup, (setup, initial_draw).chain())
    .add_systems(Update, (update_mouse_data, make_move, draw).chain())
    .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn(Camera2d);

  commands.insert_resource(Board::default());
  commands.insert_resource(MouseData::default());
  commands.insert_resource(SelectedPiece { data: None });
  commands.insert_resource(BoardAssets::new(asset_server));
}

fn initial_draw(mut commands: Commands, mut assets: ResMut<BoardAssets>, board: Res<Board>) {
  assets.draw_board(&mut commands);
  assets.draw_pieces(&mut commands, &board);
}

fn make_move(
  mouse: Res<MouseData>,
  mut board: ResMut<Board>,
  mut selected_piece: ResMut<SelectedPiece>,
) {
  let to = mouse.board_pos;

  if let Some((_, from)) = selected_piece.data {
    if mouse.just_released {
      board.move_piece(MoveInput::from_id(from, to));
      selected_piece.data = None;
    }
  }
}

fn draw(
  mut commands: Commands,
  mouse: Res<MouseData>,
  board: Res<Board>,
  mut selected_piece: ResMut<SelectedPiece>,
  mut assets: ResMut<BoardAssets>,
  mut sprite_query: Query<(Entity, &mut Transform, &PieceTag), With<PieceTag>>,
) {
  let at_mask = 1 << mouse.board_pos;

  if mouse.just_released {
    assets.remove_moves(&mut commands);
    assets.remove_pieces(&mut commands);
    assets.draw_pieces(&mut commands, &board);
  } else if mouse.just_pressed {
    let correct_turn = board.white_turn == board.is_white(at_mask) 
      || board.white_turn == board.is_black(at_mask);
    if !correct_turn {
      return;
    }

    for (entity, _, tag) in &sprite_query {
      if mouse.board_pos == tag.0 {
        selected_piece.data = Some((entity, tag.0));
        assets.draw_moves(&mut commands, &board, at_mask);
      }
    }
  } else if mouse.being_pressed {
    if let Some((entity, _)) = selected_piece.data {
      if let Ok((_, mut transform, _)) = sprite_query.get_mut(entity) {
        transform.translation.x = mouse.x;
        transform.translation.y = mouse.y;
        transform.translation.z = 1.0;
      }
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
