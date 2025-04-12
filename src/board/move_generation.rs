use super::{Board, board_movement_trait::BoardMovement, util_fns::*};

pub struct Modifier {
  flip_side: bool,
  ignore_enemy: bool,
}

impl Modifier {
  pub const NONE: &Modifier = &Modifier {
    flip_side: false,
    ignore_enemy: false,
  };

  pub const FLIP_SIDE: &Modifier = &Modifier {
    flip_side: true,
    ignore_enemy: false,
  };

  pub const NO_ENEMY_CHECK: &Modifier = &Modifier {
    flip_side: false,
    ignore_enemy: true,
  };
}

//piece moves
impl Board {
  pub(super) fn gen_pawn_default_move(&self, at_mask: u64, modifier: &Modifier) -> u64 {
    let white_turn = self.white_turn ^ modifier.flip_side;
    let pawn_default_move = branchless_if(
      white_turn,
      at_mask.move_up_mask(1),
      at_mask.move_down_mask(1),
    );

    pawn_default_move & self.empty_mask
  }

  pub(super) fn gen_pawn_advance_move(&self, at_mask: u64, modifier: &Modifier) -> u64 {
    let white_turn = self.white_turn ^ modifier.flip_side;
    let default = self.gen_pawn_default_move(at_mask, modifier);

    let can_default_mask = branchless_if(
      white_turn,
      default.move_up_mask(1),
      default.move_down_mask(1),
    );
    let can_advance_mask = branchless_if(
      white_turn,
      self.advance_mask.move_up_mask(2),
      self.advance_mask.move_down_mask(2),
    );
    let pawn_advance_move = branchless_if(
      white_turn,
      at_mask.move_up_mask(2),
      at_mask.move_down_mask(2),
    );

    pawn_advance_move & self.empty_mask & can_advance_mask & can_default_mask
  }

  pub(super) fn gen_pawn_capturing_moves(
    &self,
    at_mask: u64,
    modifier: &Modifier,
  ) -> u64 {
    let white_turn = self.white_turn ^ modifier.flip_side;

    let enemy_mask = branchless_if(
      white_turn,
      self.black.pieces_concat(),
      self.white.pieces_concat(),
    );
    let ignore_mask = mask_from_bool(modifier.ignore_enemy);

    let one_move_up = at_mask.move_up_mask(1);
    let one_move_down = at_mask.move_down_mask(1);

    let enemy_to_left = branchless_if(
      white_turn,
      one_move_up.move_left_mask(1),
      one_move_down.move_left_mask(1),
    );
    let enemy_to_right = branchless_if(
      white_turn,
      one_move_up.move_right_mask(1),
      one_move_down.move_right_mask(1),
    );

    (enemy_to_left | enemy_to_right) & (ignore_mask | enemy_mask | self.en_passant_mask)
  }

  pub(super) fn gen_knight_moves(&self, at_mask: u64, modifier: &Modifier) -> u64 {
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

  pub(super) fn gen_bishop_moves(&self, at_mask: u64, modifier: &Modifier) -> u64 {
    let mut moves = 0;

    moves |= self.gen_iterative_moves(at_mask, -1, -1, modifier);
    moves |= self.gen_iterative_moves(at_mask, 1, -1, modifier);
    moves |= self.gen_iterative_moves(at_mask, -1, 1, modifier);
    moves |= self.gen_iterative_moves(at_mask, 1, 1, modifier);

    moves
  }

  pub(super) fn gen_rook_moves(&self, at_mask: u64, modifier: &Modifier) -> u64 {
    let mut moves = 0;

    moves |= self.gen_iterative_moves(at_mask, -1, 0, modifier);
    moves |= self.gen_iterative_moves(at_mask, 0, -1, modifier);
    moves |= self.gen_iterative_moves(at_mask, 1, 0, modifier);
    moves |= self.gen_iterative_moves(at_mask, 0, 1, modifier);

    moves
  }

  pub(super) fn gen_queen_moves(&self, at_mask: u64, modifier: &Modifier) -> u64 {
    self.gen_bishop_moves(at_mask, modifier) | self.gen_rook_moves(at_mask, modifier)
  }

  pub(super) fn gen_king_moves(&self, at_mask: u64, modifier: &Modifier) -> u64 {
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

  pub(super) fn gen_offset_moves(
    &self,
    at_mask: u64,
    offsets: Vec<(i32, i32)>,
    modifier: &Modifier,
  ) -> u64 {
    let white_turn = self.white_turn ^ modifier.flip_side;
    let enemy_mask = branchless_if(
      white_turn,
      self.black.pieces_concat(),
      self.white.pieces_concat(),
    );

    let mut moves = 0;

    for (dx, dy) in offsets {
      let mut new_move = branchless_if(
        dx > 0,
        at_mask.move_right_mask(dx.abs() as u32),
        at_mask.move_left_mask(dx.abs() as u32),
      );

      new_move = branchless_if(
        dy > 0,
        new_move.move_up_mask(dy.abs() as u32),
        new_move.move_down_mask(dy.abs() as u32),
      );

      moves |= new_move & (self.empty_mask | enemy_mask);
    }

    moves
  }

  fn gen_iterative_moves(
    &self,
    at_mask: u64,
    dx: i32,
    dy: i32,
    modifier: &Modifier,
  ) -> u64 {
    let white_turn = self.white_turn ^ modifier.flip_side;
    let enemy_mask = branchless_if(
      white_turn,
      self.black.pieces_concat(),
      self.white.pieces_concat(),
    );

    let mut moves = 0;
    let mut current = (dx, dy);
    loop {
      let mut new_move = branchless_if(
        dx > 0,
        at_mask.move_right_mask(current.0.abs() as u32),
        at_mask.move_left_mask(current.0.abs() as u32),
      );
      new_move = branchless_if(
        dy > 0,
        new_move.move_up_mask(current.1.abs() as u32),
        new_move.move_down_mask(current.1.abs() as u32),
      );

      current = (current.0 + dx, current.1 + dy);

      let within_board = new_move > 0;
      let is_enemy = new_move & enemy_mask > 0;
      let is_empty = new_move & self.empty_mask > 0;
      let is_ally = !(is_empty || is_enemy);

      moves |= branchless_if(is_enemy || is_empty, new_move, 0);

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
    let enemy_mask = branchless_if(
      self.white_turn,
      self.black.pieces_concat(),
      self.white.pieces_concat(),
    );
    let white_attackers = self.white.bishops | self.white.rooks | self.white.queens;
    let black_attackers = self.black.bishops | self.black.rooks | self.black.queens;
    let correct_pieces = branchless_if(self.white_turn, black_attackers, white_attackers);

    let mut pin_path = 0;
    let mut current = (dx, dy);

    let mut ally_count = 0;
    let mut enemy_count = 0;
    let mut pinned;

    loop {
      let mut new_move = branchless_if(
        dx > 0,
        at_mask.move_right_mask(current.0.abs() as u32),
        at_mask.move_left_mask(current.0.abs() as u32),
      );
      new_move = branchless_if(
        dy > 0,
        new_move.move_up_mask(current.1.abs() as u32),
        new_move.move_down_mask(current.1.abs() as u32),
      );

      current = (current.0 + dx, current.1 + dy);

      let within_board = new_move > 0;
      let is_enemy = new_move & enemy_mask > 0;
      let is_empty = new_move & self.empty_mask > 0;
      let is_ally = !(is_empty || is_enemy);

      enemy_count += is_enemy as u64;
      ally_count += is_ally as u64;

      pinned = ally_count == 1 && enemy_count == 1 && (correct_pieces & new_move > 0);
      let done = (ally_count == 0 && enemy_count == 1)
        || (ally_count == 2 && enemy_count == 0)
        || pinned;

      pin_path |= new_move;

      if !within_board || done {
        break;
      }
    }

    branchless_if(pinned, pin_path, 0)
  }

  pub(super) fn gen_pin_filter(&self, at_mask: u64) -> u64 {
    let ally_king = branchless_if(self.white_turn, self.white.king, self.black.king);
    let mut pin_filter = 0;

    pin_filter |= self.gen_pin_path(ally_king, -1, -1);
    pin_filter |= self.gen_pin_path(ally_king, 1, -1);
    pin_filter |= self.gen_pin_path(ally_king, -1, 1);
    pin_filter |= self.gen_pin_path(ally_king, 1, 1);
    pin_filter |= self.gen_pin_path(ally_king, -1, 0);
    pin_filter |= self.gen_pin_path(ally_king, 0, -1);
    pin_filter |= self.gen_pin_path(ally_king, 1, 0);
    pin_filter |= self.gen_pin_path(ally_king, 0, 1);

    branchless_if(pin_filter & at_mask > 0, pin_filter, u64::MAX)
  }

  pub(super) fn gen_check_filter(&self, at_mask: u64) -> u64 {
    let ally_king = branchless_if(self.white_turn, self.white.king, self.black.king);
    let enemy_pawns = branchless_if(self.white_turn, self.black.pawns, self.white.pawns);
    let enemy_knights =
      branchless_if(self.white_turn, self.black.knights, self.white.knights);
    let enemy_bishops =
      branchless_if(self.white_turn, self.black.bishops, self.white.bishops);
    let enemy_rooks = branchless_if(self.white_turn, self.black.rooks, self.white.rooks);
    let enemy_queens =
      branchless_if(self.white_turn, self.black.queens, self.white.queens);

    let mut filter = 0;
    filter |= self.gen_pawn_capturing_moves(ally_king, Modifier::NONE) & enemy_pawns;
    filter |= self.gen_knight_moves(ally_king, Modifier::NONE) & enemy_knights;
    filter |= branchless_if(at_mask == ally_king, u64::MAX, 0);

    let bishop_attacks =
      self.gen_bishop_moves(enemy_bishops | enemy_queens, Modifier::FLIP_SIDE) | enemy_bishops | enemy_queens;
    let king_as_bishop = self.gen_bishop_moves(ally_king, Modifier::NONE) | ally_king;
    let combined = bishop_attacks & king_as_bishop;
    filter |= branchless_if(combined & ally_king > 0, combined, 0);

    let rook_attacks =
      self.gen_rook_moves(enemy_rooks | enemy_queens, Modifier::FLIP_SIDE) | enemy_rooks | enemy_queens;
    let king_as_rook = self.gen_rook_moves(ally_king, Modifier::NONE) | ally_king;
    let combined = rook_attacks & king_as_rook;
    filter |= branchless_if(combined & ally_king > 0, combined, 0);

    branchless_if(filter > 0, filter, u64::MAX)
  }
}
