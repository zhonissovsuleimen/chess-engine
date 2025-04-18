use super::{
  Board,
  board_movement_trait::BoardMovement,
  move_generation_modifiers::{ALLY_IS_ENEMY, NONE, ally_is_enemy, empty_is_enemy, king_is_empty},
  util_fns::*,
};

//piece moves
impl Board {
  pub(super) fn gen_pawn_default_move(&self, at_mask: u64) -> u64 {
    let pawn_default_move = if_mask(
      self.white_turn_mask,
      at_mask.move_up_mask(1),
      at_mask.move_down_mask(1),
    );

    pawn_default_move & self.empty
  }

  pub(super) fn gen_pawn_advance_move(&self, at_mask: u64) -> u64 {
    let default = self.gen_pawn_default_move(at_mask);

    let can_default_mask = if_mask(
      self.white_turn_mask,
      default.move_up_mask(1),
      default.move_down_mask(1),
    );
    let can_advance_mask = if_mask(
      self.white_turn_mask,
      self.advance_mask.move_up_mask(2),
      self.advance_mask.move_down_mask(2),
    );
    let pawn_advance_move = if_mask(
      self.white_turn_mask,
      at_mask.move_up_mask(2),
      at_mask.move_down_mask(2),
    );

    pawn_advance_move & self.empty & can_advance_mask & can_default_mask
  }

  pub(super) fn gen_pawn_capturing_moves(&self, at_mask: u64, modifier: u64) -> u64 {
    let mut empty = self.empty;
    empty |= mask_from_bool(king_is_empty(modifier)) & self.ally_king;

    let mut enemy = self.enemy;
    enemy |= mask_from_bool(empty_is_enemy(modifier)) & empty;
    enemy |= mask_from_bool(ally_is_enemy(modifier)) & self.ally;
    enemy &= !(mask_from_bool(king_is_empty(modifier)) & self.ally_king);

    let one_move_up = at_mask.move_up_mask(1);
    let one_move_down = at_mask.move_down_mask(1);

    let enemy_to_left = if_mask(
      self.white_turn_mask,
      one_move_up.move_left_mask(1),
      one_move_down.move_left_mask(1),
    );
    let enemy_to_right = if_mask(
      self.white_turn_mask,
      one_move_up.move_right_mask(1),
      one_move_down.move_right_mask(1),
    );

    (enemy_to_left | enemy_to_right) & (enemy | self.en_passant_mask)
  }

  pub(super) fn gen_knight_moves(&self, at_mask: u64, modifier: u64) -> u64 {
    let offsets = [
      (-1, -2),
      (1, -2),
      (-2, -1),
      (2, -1),
      (-2, 1),
      (2, 1),
      (-1, 2),
      (1, 2),
    ]
    .to_vec();

    self.gen_offset_moves(at_mask, offsets, modifier)
  }

  pub(super) fn gen_bishop_moves(&self, at_mask: u64, modifier: u64) -> u64 {
    let mut moves = 0;

    moves |= self.gen_iterative_moves(at_mask, -1, -1, modifier);
    moves |= self.gen_iterative_moves(at_mask, 1, -1, modifier);
    moves |= self.gen_iterative_moves(at_mask, -1, 1, modifier);
    moves |= self.gen_iterative_moves(at_mask, 1, 1, modifier);

    moves
  }

  pub(super) fn gen_rook_moves(&self, at_mask: u64, modifier: u64) -> u64 {
    let mut moves = 0;

    moves |= self.gen_iterative_moves(at_mask, -1, 0, modifier);
    moves |= self.gen_iterative_moves(at_mask, 0, -1, modifier);
    moves |= self.gen_iterative_moves(at_mask, 1, 0, modifier);
    moves |= self.gen_iterative_moves(at_mask, 0, 1, modifier);

    moves
  }

  pub(super) fn gen_queen_moves(&self, at_mask: u64, modifier: u64) -> u64 {
    self.gen_bishop_moves(at_mask, modifier) | self.gen_rook_moves(at_mask, modifier)
  }

  pub(super) fn gen_king_default_moves(&self, at_mask: u64, modifier: u64) -> u64 {
    let offsets = [
      (-1, -1),
      (0, -1),
      (1, -1),
      (-1, 0),
      (1, 0),
      (-1, 1),
      (0, 1),
      (1, 1),
    ]
    .to_vec();

    self.gen_offset_moves(at_mask, offsets, modifier)
  }

  pub(super) fn gen_king_long_castle_moves(&self, at_mask: u64) -> u64 {
    let rooks = self.rooks() | self.ally;
    let long_rook = rooks & at_mask.move_left_mask(4);

    let long_rights = self.castling_mask & long_rook > 0 && self.castling_mask & at_mask > 0;
    let long_empty = at_mask
      & self.empty.move_right_mask(1)
      & self.empty.move_right_mask(2)
      & self.empty.move_right_mask(3)
      > 0;
    let long_safe = at_mask
      & !self.king_danger
      & !self.king_danger.move_right_mask(1)
      & !self.king_danger.move_right_mask(2)
      > 0;

    let new_pos = at_mask.move_left_mask(2);
    new_pos & mask_from_bool(long_rights && long_empty && long_safe)
  }

  pub(super) fn gen_king_short_castle_moves(&self, at_mask: u64) -> u64 {
    let rooks = self.rooks() | self.ally;
    let short_rook = rooks & at_mask.move_right_mask(3);

    let short_rights = self.castling_mask & short_rook > 0 && self.castling_mask & at_mask > 0;
    let short_empty = at_mask & self.empty.move_left_mask(1) & self.empty.move_left_mask(2) > 0;
    let short_safe = at_mask
      & !self.king_danger
      & self.king_danger.move_left_mask(1)
      & !self.king_danger.move_left_mask(2)
      > 0;

    let new_pos = at_mask.move_right_mask(2);
    new_pos & mask_from_bool(short_rights && short_empty && short_safe)
  }

  pub(super) fn gen_offset_moves(
    &self,
    at_mask: u64,
    offsets: Vec<(i32, i32)>,
    modifier: u64,
  ) -> u64 {
    let mut empty = self.empty;
    empty |= mask_from_bool(king_is_empty(modifier)) & self.ally_king;

    let mut enemy = self.enemy;
    enemy |= mask_from_bool(empty_is_enemy(modifier)) & empty;
    enemy |= mask_from_bool(ally_is_enemy(modifier)) & self.ally;
    enemy &= !(mask_from_bool(king_is_empty(modifier)) & self.ally_king);

    let mut moves = 0;

    for (dx, dy) in offsets {
      let mut new_move = if_bool(
        dx > 0,
        at_mask.move_right_mask(dx.unsigned_abs()),
        at_mask.move_left_mask(dx.unsigned_abs()),
      );

      new_move = if_bool(
        dy > 0,
        new_move.move_up_mask(dy.unsigned_abs()),
        new_move.move_down_mask(dy.unsigned_abs()),
      );

      moves |= new_move & (empty | enemy);
    }

    moves
  }

  fn gen_iterative_moves(&self, at_mask: u64, dx: i32, dy: i32, modifier: u64) -> u64 {
    let mut empty = self.empty;
    empty |= mask_from_bool(king_is_empty(modifier)) & self.ally_king;

    let mut enemy = self.enemy;
    enemy |= mask_from_bool(empty_is_enemy(modifier)) & empty;
    enemy |= mask_from_bool(ally_is_enemy(modifier)) & self.ally;
    enemy &= !(mask_from_bool(king_is_empty(modifier)) & self.ally_king);

    let mut moves = 0;
    let mut current = (dx, dy);
    loop {
      let mut new_move = if_bool(
        dx > 0,
        at_mask.move_right_mask(current.0.unsigned_abs()),
        at_mask.move_left_mask(current.0.unsigned_abs()),
      );
      new_move = if_bool(
        dy > 0,
        new_move.move_up_mask(current.1.unsigned_abs()),
        new_move.move_down_mask(current.1.unsigned_abs()),
      );

      current = (current.0 + dx, current.1 + dy);

      let within_board = new_move > 0;
      let is_enemy = new_move & enemy > 0;
      let is_empty = new_move & empty > 0;
      let is_ally = !(is_empty || is_enemy);

      moves |= if_bool(is_enemy || is_empty, new_move, 0);

      if !within_board || is_ally || is_enemy {
        break;
      }
    }

    moves
  }
}

//other
impl Board {
  pub(super) fn gen_pin_filter(&self, at_mask: u64) -> u64 {
    let directions = [
      (-1, 1, u64::MAX),
      (0, 1, 0),
      (1, 1, u64::MAX),
      (-1, 0, 0),
      (1, 0, 0),
      (-1, -1, u64::MAX),
      (0, -1, 0),
      (1, -1, u64::MAX),
    ];

    let mut pin_path = 0;
    let diag = self.enemy & (self.bishops() | self.queens());
    let not_diag = self.enemy & (self.rooks() | self.queens());

    for (dx, dy, diag_mask) in directions {
      let pinner = if_mask(diag_mask, diag, not_diag);

      let king_to_pin_path = self.gen_iterative_moves(self.ally_king, dx, dy, ALLY_IS_ENEMY);
      let king_hits_pin = king_to_pin_path & at_mask > 0;

      let pin_to_pinner_path = self.gen_iterative_moves(at_mask, dx, dy, NONE) | pinner;
      let pin_hits_pinner = pin_to_pinner_path & pinner > 0;

      let pinned = mask_from_bool(king_hits_pin & pin_hits_pinner);
      pin_path = pinned & (king_to_pin_path | pin_to_pinner_path);
    }

    //king not under attack? kinda discovered pin and king attack
    if_bool(pin_path > 0, pin_path, u64::MAX)
  }

  pub(super) fn gen_check_filter(&self, at_mask: u64) -> u64 {
    let enemy_pawns = self.pawns() & self.enemy;
    let enemy_knights = self.knights() & self.enemy;

    let mut filter = 0;
    filter |= self.gen_pawn_capturing_moves(self.ally_king, NONE) & enemy_pawns;
    filter |= self.gen_knight_moves(self.ally_king, NONE) & enemy_knights;
    filter |= mask_from_bool(at_mask == self.ally_king);

    let directions = [
      (-1, 1, u64::MAX),
      (0, 1, 0),
      (1, 1, u64::MAX),
      (-1, 0, 0),
      (1, 0, 0),
      (-1, -1, u64::MAX),
      (0, -1, 0),
      (1, -1, u64::MAX),
    ];

    let diag = self.enemy & (self.bishops() | self.queens());
    let not_diag = self.enemy & (self.rooks() | self.queens());

    for (dx, dy, diag_mask) in directions {
      let attacker = if_mask(diag_mask, diag, not_diag);

      let king_to_attacker_path = self.gen_iterative_moves(self.ally_king, dx, dy, NONE);
      let king_hits_attacker = mask_from_bool(king_to_attacker_path & attacker > 0);

      filter |= king_hits_attacker & (king_to_attacker_path | attacker);
    }

    filter = mask_from_bool(at_mask == self.ally_king) ^ filter;

    if_bool(filter > 0, filter, u64::MAX)
  }
}
