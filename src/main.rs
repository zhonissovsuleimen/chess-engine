mod board;
mod board_assets;
mod board_position_lookup;

use bevy::{prelude::*, window::PrimaryWindow};
use board::{move_input::MoveInput, status::PLAYING, Board};
use board_assets::{BoardAssets, PieceTag, PromotionTag};
use board_position_lookup::{CENTER_LOOKUP, X_LOOKUP, Y_LOOKUP};

#[derive(Resource, Default)]
enum DrawMode {
  #[default]
  SelectPiece,
  DrawMoves,
  DragPiece,
  DrawPromotion,
  SelectPromotion,
  MakeMove,
  Reset,
}

#[derive(Resource, Default)]
struct State {
  mode: DrawMode,
  entity: Option<Entity>,
  selected_from: Option<usize>,
  selected_to: Option<usize>,
  selected_promotion: Option<u64>,
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
    .add_systems(Update, (update_mouse_data, update_state, draw).chain())
    .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn(Camera2d);

  commands.insert_resource(Board::default());
  commands.insert_resource(MouseData::default());
  commands.insert_resource(BoardAssets::new(asset_server));
  commands.insert_resource(State::default());
}

fn initial_draw(mut commands: Commands, mut assets: ResMut<BoardAssets>, board: Res<Board>) {
  assets.draw_board(&mut commands);
  assets.draw_pieces(&mut commands, &board);
}

fn draw(
  mut commands: Commands,
  mouse: Res<MouseData>,
  board: Res<Board>,
  state: Res<State>,
  mut assets: ResMut<BoardAssets>,
  mut sprite_query: Query<(Entity, &mut Transform, &PieceTag), With<PieceTag>>,
) {
  match state.mode {
    DrawMode::DrawMoves => {
      assets.draw_moves(&mut commands, &board);
    }
    DrawMode::DragPiece => {
      if let Some(entity) = state.entity {
        if let Ok((_, mut transform, _)) = sprite_query.get_mut(entity) {
          transform.translation.x = mouse.x;
          transform.translation.y = mouse.y;
          transform.translation.z = 1.0;
        }
      }
    }
    DrawMode::DrawPromotion => {
      if let Some(entity) = state.entity {
        if let Ok((_, mut transform, _)) = sprite_query.get_mut(entity) {
          transform.translation.z = -transform.translation.z;
        }
      }

      if let Some(from_mask) = state.selected_from {
        assets.draw_promotion(&mut commands, &board, from_mask);
      }
    }
    DrawMode::Reset => {
      assets.remove_moves(&mut commands);
      assets.remove_promotion(&mut commands);
      assets.remove_pieces(&mut commands);
      assets.draw_pieces(&mut commands, &board);
    }
    _ => {}
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

fn update_state(
  mouse: Res<MouseData>,
  mut state: ResMut<State>,
  mut board: ResMut<Board>,
  pieces: Query<(Entity, &mut Transform), With<PieceTag>>,
  promotions: Query<(&Transform, &PromotionTag), (With<PromotionTag>, Without<PieceTag>)>,
) {
  match state.mode {
    DrawMode::SelectPiece if mouse.just_pressed => {
      state.selected_from = Some(mouse.board_pos);
      let from_mask = 1 << mouse.board_pos;
      board.update_cache(from_mask);

      for (entity, transform) in pieces.iter() {
        if transform.translation.truncate() == CENTER_LOOKUP[mouse.board_pos].truncate() {
          state.entity = Some(entity);
          break;
        }
      }
      state.mode = DrawMode::DrawMoves;
    }
    DrawMode::DrawMoves => {
      state.mode = DrawMode::DragPiece;
    }
    DrawMode::DragPiece if mouse.just_released => {
      state.selected_to = Some(mouse.board_pos);

      let to_mask = 1 << mouse.board_pos;
      if board.is_promotion(to_mask) {
        state.mode = DrawMode::DrawPromotion;
      } else {
        state.mode = DrawMode::MakeMove;
        state.selected_promotion = Some(0);
      }
    }
    DrawMode::DrawPromotion => {
      state.mode = DrawMode::SelectPromotion;
    }
    DrawMode::SelectPromotion if mouse.just_released => {
      for (transform, tag) in promotions.iter() {
        if transform.translation.truncate() == CENTER_LOOKUP[mouse.board_pos].truncate() {
          state.selected_promotion = Some(tag.0);
        }
      }

      if state.selected_promotion.is_some() {
        state.mode = DrawMode::MakeMove;
      } else {
        state.mode = DrawMode::Reset;
      }
    }
    DrawMode::MakeMove => {
      if let Some(from) = state.selected_from {
        if let Some(to) = state.selected_to {
          if let Some(promotion) = state.selected_promotion {
            board.move_piece(MoveInput {
              from: 1 << from,
              to: 1 << to,
              promotion,
            });

            if board.get_status() != PLAYING {
              *board = Board::default();
            }
          }
        }
      }
      state.mode = DrawMode::Reset;
    }
    DrawMode::Reset => {
      std::mem::take(&mut *state);
    }
    _ => {}
  };
}
