use crate::{
  board::{
    Board,
    move_input::{BISHOP, KNIGHT, QUEEN, ROOK},
  },
  board_position_lookup::CENTER_LOOKUP,
};
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
pub struct PieceTag;

#[derive(Component)]
pub struct PromotionTag(pub u64);

const BOARD_Z: f32 = 0.0;
const PIECES_Z: f32 = 0.1;
const MOVES_Z: f32 = 0.2;
const PROMOTION_Z: f32 = 0.3;

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
  promotion_ids: Vec<Entity>,

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
      promotion_ids: Vec::new(),

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
    let transform = Transform {
      translation: Vec3::new(0.0, 0.0, BOARD_Z),
      scale: Vec3::ONE,
      rotation: Quat::IDENTITY,
    };

    commands.spawn((Sprite::from_image(self.board.clone()), transform));
  }

  pub fn draw_pieces(&mut self, commands: &mut Commands, board: &Board) {
    for (i, lookup) in CENTER_LOOKUP.into_iter().enumerate() {
      let at_mask = 1 << i;
      if board.is_empty(at_mask) {
        continue;
      }

      let transform = Transform {
        translation: lookup + Vec3::new(0.0, 0.0, PIECES_Z),
        scale: Vec3::new(0.75, 0.75, 1.0),
        rotation: Quat::IDENTITY,
      };

      let (pieces, assets) = if board.white.is_empty(at_mask) {
        (&board.black, &self.black)
      } else {
        (&board.white, &self.white)
      };

      if pieces.is_pawn(at_mask) {
        self.piece_ids.push(
          commands
            .spawn((Sprite::from_image(assets.pawn.clone()), transform, PieceTag))
            .id(),
        );
      } else if pieces.is_knight(at_mask) {
        self.piece_ids.push(
          commands
            .spawn((
              Sprite::from_image(assets.knight.clone()),
              transform,
              PieceTag,
            ))
            .id(),
        );
      } else if pieces.is_bishop(at_mask) {
        self.piece_ids.push(
          commands
            .spawn((
              Sprite::from_image(assets.bishop.clone()),
              transform,
              PieceTag,
            ))
            .id(),
        );
      } else if pieces.is_rook(at_mask) {
        self.piece_ids.push(
          commands
            .spawn((Sprite::from_image(assets.rook.clone()), transform, PieceTag))
            .id(),
        );
      } else if pieces.is_queen(at_mask) {
        self.piece_ids.push(
          commands
            .spawn((
              Sprite::from_image(assets.queen.clone()),
              transform,
              PieceTag,
            ))
            .id(),
        );
      } else if pieces.is_king(at_mask) {
        self.piece_ids.push(
          commands
            .spawn((Sprite::from_image(assets.king.clone()), transform, PieceTag))
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

  pub fn draw_moves(&mut self, commands: &mut Commands, board: &Board) {
    let moves = board.cached_moves.all();

    for (i, lookup) in CENTER_LOOKUP.into_iter().enumerate() {
      if moves & (1 << i) == 0 {
        continue;
      }

      let transform = Transform {
        translation: lookup + Vec3::new(0.0, 0.0, MOVES_Z),
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

  pub fn draw_promotion(&mut self, commands: &mut Commands, board: &Board, from_id: usize) {
    let (assets, offset) = if board.white_turn {
      (&self.white, -96.0)
    } else {
      (&self.black, 96.0)
    };

    let queen = (
      Sprite::from(assets.queen.clone()),
      Transform {
        translation: CENTER_LOOKUP[from_id] + Vec3::new(0.0, -offset, PROMOTION_Z),
        scale: Vec3::new(0.75, 0.75, 1.0),
        rotation: Quat::IDENTITY,
      },
      PromotionTag(QUEEN),
    );

    let rook = (
      Sprite::from(assets.rook.clone()),
      Transform {
        translation: CENTER_LOOKUP[from_id] + Vec3::new(0.0, 0.0, 0.1),
        scale: Vec3::new(0.75, 0.75, 1.0),
        rotation: Quat::IDENTITY,
      },
      PromotionTag(ROOK),
    );

    let bishop = (
      Sprite::from(assets.bishop.clone()),
      Transform {
        translation: CENTER_LOOKUP[from_id] + Vec3::new(0.0, offset, 0.1),
        scale: Vec3::new(0.75, 0.75, 1.0),
        rotation: Quat::IDENTITY,
      },
      PromotionTag(BISHOP),
    );

    let knight = (
      Sprite::from(assets.knight.clone()),
      Transform {
        translation: CENTER_LOOKUP[from_id] + Vec3::new(0.0, 2.0 * offset, 0.1),
        scale: Vec3::new(0.75, 0.75, 1.0),
        rotation: Quat::IDENTITY,
      },
      PromotionTag(KNIGHT),
    );

    self.promotion_ids.push(commands.spawn(queen).id());
    self.promotion_ids.push(commands.spawn(rook).id());
    self.promotion_ids.push(commands.spawn(bishop).id());
    self.promotion_ids.push(commands.spawn(knight).id());
  }

  pub fn remove_promotion(&mut self, commands: &mut Commands) {
    for &id in self.promotion_ids.iter() {
      if let Some(mut entity) = commands.get_entity(id) {
        entity.despawn();
      }
    }
  }
}
