use super::{
  Board,
  board_movement_trait::BoardMovement,
  move_generation_modifiers::{ALLY_IS_ENEMY, NONE, ally_is_enemy, empty_is_enemy},
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
    let mut enemy_mask = self.enemy;
    enemy_mask |= mask_from_bool(empty_is_enemy(modifier)) & self.empty;
    enemy_mask |= mask_from_bool(ally_is_enemy(modifier)) & self.ally;

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

    (enemy_to_left | enemy_to_right) & (enemy_mask | self.en_passant_mask)
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
      & self.empty.move_right_mask(3) > 0;
    let long_safe = at_mask & !self.under_attack & !self.under_attack.move_right_mask(2) > 0;

    let new_pos = at_mask.move_left_mask(2);
    new_pos & mask_from_bool(long_rights && long_empty && long_safe)
  }

  pub(super) fn gen_king_short_castle_moves(&self, at_mask: u64) -> u64 {
    let rooks = self.rooks() | self.ally;
    let short_rook = rooks & at_mask.move_right_mask(3);

    let short_rights = self.castling_mask & short_rook > 0 && self.castling_mask & at_mask > 0;
    let short_empty = at_mask 
      & self.empty.move_left_mask(1) 
      & self.empty.move_left_mask(2) > 0;
    let short_safe = at_mask & !self.under_attack & !self.under_attack.move_left_mask(2) > 0;

    let new_pos = at_mask.move_right_mask(2);
    new_pos & mask_from_bool(short_rights && short_empty && short_safe)
  }

  pub(super) fn gen_offset_moves(
    &self,
    at_mask: u64,
    offsets: Vec<(i32, i32)>,
    modifier: u64,
  ) -> u64 {
    let mut enemy_mask = self.enemy;
    enemy_mask |= mask_from_bool(empty_is_enemy(modifier)) & self.empty;
    enemy_mask |= mask_from_bool(ally_is_enemy(modifier)) & self.ally;

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

      moves |= new_move & (self.empty | enemy_mask);
    }

    moves
  }

  fn gen_iterative_moves(&self, at_mask: u64, dx: i32, dy: i32, modifier: u64) -> u64 {
    let mut enemy_mask = self.enemy;
    enemy_mask |= mask_from_bool(empty_is_enemy(modifier)) & self.empty;
    enemy_mask |= mask_from_bool(ally_is_enemy(modifier)) & self.ally;

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
      let is_enemy = new_move & enemy_mask > 0;
      let is_empty = new_move & self.empty > 0;
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
  fn gen_pin_path(&self, at_mask: u64, dx: i32, dy: i32) -> u64 {
    let white_attackers = self.white.bishops | self.white.rooks | self.white.queens;
    let black_attackers = self.black.bishops | self.black.rooks | self.black.queens;
    let correct_pieces = if_mask(self.white_turn_mask, black_attackers, white_attackers);

    let mut pin_path = 0;
    let mut current = (dx, dy);

    let mut ally_count = 0;
    let mut enemy_count = 0;
    let mut pinned;

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
      let is_enemy = new_move & self.enemy > 0;
      let is_ally = new_move & self.ally > 0;

      enemy_count += is_enemy as u64;
      ally_count += is_ally as u64;

      pinned = ally_count == 1 && enemy_count == 1 && (correct_pieces & new_move > 0);
      let done =
        (ally_count == 0 && enemy_count == 1) || (ally_count == 2 && enemy_count == 0) || pinned;

      pin_path |= new_move;

      if !within_board || done {
        break;
      }
    }

    if_bool(pinned, pin_path, 0)
  }

  pub(super) fn gen_pin_filter(&self, at_mask: u64) -> u64 {
    let mut pin_filter = 0;

    pin_filter |= self.gen_pin_path(self.ally_king, -1, -1);
    pin_filter |= self.gen_pin_path(self.ally_king, 1, -1);
    pin_filter |= self.gen_pin_path(self.ally_king, -1, 1);
    pin_filter |= self.gen_pin_path(self.ally_king, 1, 1);
    pin_filter |= self.gen_pin_path(self.ally_king, -1, 0);
    pin_filter |= self.gen_pin_path(self.ally_king, 0, -1);
    pin_filter |= self.gen_pin_path(self.ally_king, 1, 0);
    pin_filter |= self.gen_pin_path(self.ally_king, 0, 1);

    if_bool(pin_filter & at_mask > 0, pin_filter, u64::MAX)
  }

  pub(super) fn gen_check_filter(&self, at_mask: u64) -> u64 {
    let enemy_pawns = if_mask(self.white_turn_mask, self.black.pawns, self.white.pawns);
    let enemy_knights = if_mask(self.white_turn_mask, self.black.knights, self.white.knights);
    let enemy_bishops = if_mask(self.white_turn_mask, self.black.bishops, self.white.bishops);
    let enemy_rooks = if_mask(self.white_turn_mask, self.black.rooks, self.white.rooks);
    let enemy_queens = if_mask(self.white_turn_mask, self.black.queens, self.white.queens);

    let mut filter = 0;
    filter |= self.gen_pawn_capturing_moves(self.ally_king, NONE) & enemy_pawns;
    filter |= self.gen_knight_moves(self.ally_king, NONE) & enemy_knights;
    filter |= if_bool(at_mask == self.ally_king, u64::MAX, 0);

    let bishop_attacks = self.gen_bishop_moves(enemy_bishops | enemy_queens, ALLY_IS_ENEMY)
      | enemy_bishops
      | enemy_queens;
    let king_as_bishop = self.gen_bishop_moves(self.ally_king, NONE) | self.ally_king;
    let combined = bishop_attacks & king_as_bishop;
    filter |= if_bool(combined & self.ally_king > 0, combined, 0);

    let rook_attacks =
      self.gen_rook_moves(enemy_rooks | enemy_queens, ALLY_IS_ENEMY) | enemy_rooks | enemy_queens;
    let king_as_rook = self.gen_rook_moves(self.ally_king, NONE) | self.ally_king;
    let combined = rook_attacks & king_as_rook;
    filter |= if_bool(combined & self.ally_king > 0, combined, 0);

    if_bool(filter > 0, filter, u64::MAX)
  }
}
