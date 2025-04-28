use crate::{board::Board, board_position_lookup::CENTER_LOOKUP};
use bevy::{
  asset::{AssetServer, Handle},
  color::Alpha,
  ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, Res, Resource},
  },
  image::Image,
  math::{Quat, Vec3},
  sprite::Sprite,
  transform::components::Transform,
};

#[derive(Component)]
pub struct PieceTag(pub usize);

pub struct PiecesAssets {
  pawn: Handle<Image>,
  knight: Handle<Image>,
  bishop: Handle<Image>,
  rook: Handle<Image>,
  queen: Handle<Image>,
  king: Handle<Image>,
}

#[derive(Resource)]
pub struct BoardAssets {
  piece_ids: Vec<Entity>,
  move_ids: Vec<Entity>,

  board: Handle<Image>,
  circle: Handle<Image>,
  white: PiecesAssets,
  black: PiecesAssets,
}

impl BoardAssets {
  pub fn new(server: Res<AssetServer>) -> BoardAssets {
    BoardAssets {
      piece_ids: Vec::new(),
      move_ids: Vec::new(),

      board: server.load("board.png"),
      circle: server.load("circle.png"),
      white: PiecesAssets {
        pawn: server.load(r"pieces\white-pawn.png"),
        knight: server.load(r"pieces\white-knight.png"),
        bishop: server.load(r"pieces\white-bishop.png"),
        rook: server.load(r"pieces\white-rook.png"),
        queen: server.load(r"pieces\white-queen.png"),
        king: server.load(r"pieces\white-king.png"),
      },
      black: PiecesAssets {
        pawn: server.load(r"pieces\black-pawn.png"),
        knight: server.load(r"pieces\black-knight.png"),
        bishop: server.load(r"pieces\black-bishop.png"),
        rook: server.load(r"pieces\black-rook.png"),
        queen: server.load(r"pieces\black-queen.png"),
        king: server.load(r"pieces\black-king.png"),
      },
    }
  }

  pub fn draw_board(&self, commands: &mut Commands) {
    commands.spawn(Sprite::from_image(self.board.clone()));
  }

  pub fn draw_pieces(&mut self, commands: &mut Commands, board: &Res<Board>) {
    for (i, lookup) in CENTER_LOOKUP.into_iter().enumerate() {
      let at_mask = 1 << i;
      if board.is_empty(at_mask) {
        continue;
      }

      let transform = Transform {
        translation: lookup,
        scale: Vec3::new(0.75, 0.75, 1.0),
        rotation: Quat::IDENTITY,
      };
      let tag = PieceTag(i);

      let (pieces, assets) = if board.white.is_empty(at_mask) {
        (&board.black, &self.black)
      } else {
        (&board.white, &self.white)
      };

      if pieces.is_pawn(at_mask) {
        self.piece_ids.push(
          commands
            .spawn((Sprite::from_image(assets.pawn.clone()), transform, tag))
            .id(),
        );
      } else if pieces.is_knight(at_mask) {
        self.piece_ids.push(
          commands
            .spawn((Sprite::from_image(assets.knight.clone()), transform, tag))
            .id(),
        );
      } else if pieces.is_bishop(at_mask) {
        self.piece_ids.push(
          commands
            .spawn((Sprite::from_image(assets.bishop.clone()), transform, tag))
            .id(),
        );
      } else if pieces.is_rook(at_mask) {
        self.piece_ids.push(
          commands
            .spawn((Sprite::from_image(assets.rook.clone()), transform, tag))
            .id(),
        );
      } else if pieces.is_queen(at_mask) {
        self.piece_ids.push(
          commands
            .spawn((Sprite::from_image(assets.queen.clone()), transform, tag))
            .id(),
        );
      } else if pieces.is_king(at_mask) {
        self.piece_ids.push(
          commands
            .spawn((Sprite::from_image(assets.king.clone()), transform, tag))
            .id(),
        );
      }
    }
  }

  pub fn remove_pieces(&self, commands: &mut Commands) {
    for id in &self.piece_ids {
      if let Some(mut entity) = commands.get_entity(*id) {
        entity.despawn();
      }
    }
  }

  pub fn draw_moves(&mut self, commands: &mut Commands, board: &Res<Board>, at_mask: u64) {
    let moves = board.get_piece_moves(at_mask);

    for (i, lookup) in CENTER_LOOKUP.into_iter().enumerate() {
      if moves & (1 << i) == 0 {
        continue;
      }

      let transform = Transform {
        translation: lookup + Vec3::new(0.0, 0.0, 0.1),
        scale: Vec3::new(0.2, 0.2, 1.0),
        rotation: Quat::IDENTITY,
      };
      let mut sprite = Sprite::from_image(self.circle.clone());
      sprite.color.set_alpha(0.5);

      self.move_ids.push(commands.spawn((sprite, transform)).id());
    }
  }

  pub fn remove_moves(&self, commands: &mut Commands) {
    for id in &self.move_ids {
      if let Some(mut entity) = commands.get_entity(*id) {
        entity.despawn();
      }
    }
  }
}
