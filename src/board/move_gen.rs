use std::mem;

use super::{
  Board, board_movement_trait::BoardMovement, cached_piece_moves::CachedPieceMoves, status::*,
  util_fns::*,
};

pub struct MoveGen {
  default_white_turn_mask: u64,
  default_ally: u64,
  default_enemy: u64,
  default_empty: u64,
  saved_white_turn_mask: u64,
  saved_ally: u64,
  saved_enemy: u64,
  saved_empty: u64,

  white_turn_mask: u64,
  ally: u64,
  enemy: u64,
  empty: u64,
  pawns: u64,
  knights: u64,
  bishops: u64,
  rooks: u64,
  queens: u64,
  kings: u64,

  advance_mask: u64,
  en_passant_mask: u64,
  castling_mask: u64,
  king_danger: u64,
}

//constructors
impl MoveGen {
  pub(super) fn default(board: &Board) -> MoveGen {
    let white_turn_mask = mask_from_bool(board.white_turn);
    let ally = if_mask(
      white_turn_mask,
      board.white.pieces_concat(),
      board.black.pieces_concat(),
    );
    let enemy = if_mask(
      white_turn_mask,
      board.black.pieces_concat(),
      board.white.pieces_concat(),
    );
    let empty = !(ally | enemy);

    let mut movegen = MoveGen {
      default_white_turn_mask: white_turn_mask,
      default_ally: ally,
      default_enemy: enemy,
      default_empty: empty,
      saved_white_turn_mask: white_turn_mask,
      saved_ally: ally,
      saved_enemy: enemy,
      saved_empty: empty,

      white_turn_mask: white_turn_mask,
      ally: ally,
      enemy: enemy,
      empty: empty,
      pawns: board.pawns(),
      knights: board.knights(),
      bishops: board.bishops(),
      rooks: board.rooks(),
      queens: board.queens(),
      kings: board.kings(),

      advance_mask: board.advance_mask,
      en_passant_mask: board.en_passant_mask,
      castling_mask: board.castling_mask,
      king_danger: 0,
    };

    movegen.switch_turn();
    movegen.skip_enemy_king();
    movegen.ally_is_enemy();
    movegen.king_danger = movegen.knight(movegen.knights & movegen.enemy)
      | movegen.bishop(movegen.bishops & movegen.enemy)
      | movegen.rook(movegen.rooks & movegen.enemy)
      | movegen.queen(movegen.queens & movegen.enemy)
      | movegen.king_default(movegen.kings & movegen.enemy);
    movegen.reset();

    movegen.switch_turn();
    movegen.empty_is_enemy();
    movegen.king_danger |= movegen.pawn_capturing(movegen.pawns & movegen.enemy);
    movegen.reset();
    movegen
  }

  pub(super) fn cached(board: &Board, at_mask: u64) -> CachedPieceMoves {
    let mut movegen = MoveGen::default(board);
    let pin = movegen.pin_filter(at_mask);
    let check = movegen.check_filter(at_mask);
    let filter = pin & check;

    let mut moves = CachedPieceMoves {
      pawn_default: movegen.pawn_default(at_mask & movegen.pawns & movegen.ally) & filter,
      pawn_advance: movegen.pawn_advance(at_mask & movegen.pawns & movegen.ally) & filter,
      pawn_capturing: movegen.pawn_capturing(at_mask & movegen.pawns & movegen.ally) & filter,
      knight: movegen.knight(at_mask & movegen.knights & movegen.ally) & filter,
      bishop: movegen.bishop(at_mask & movegen.bishops & movegen.ally) & filter,
      rook: movegen.rook(at_mask & movegen.rooks & movegen.ally) & filter,
      queen: movegen.queen(at_mask & movegen.queens & movegen.ally) & filter,
      king_default: movegen.king_default(at_mask & movegen.kings & movegen.ally)
        & !movegen.king_danger
        & filter,
      king_short_castle: movegen.king_short_castle(at_mask & movegen.kings & movegen.ally),
      king_long_castle: movegen.king_long_castle(at_mask & movegen.kings & movegen.ally),
      capturing: 0,
      status: 0,
    };
    moves.capturing = moves.all() & movegen.enemy;
    moves.status = movegen.status();
    moves
  }
}

impl MoveGen {
  fn pawn_default(&self, at_mask: u64) -> u64 {
    let default = if_mask(
      self.white_turn_mask,
      at_mask.move_up_mask(1),
      at_mask.move_down_mask(1),
    );

    default & self.empty
  }

  fn pawn_advance(&self, at_mask: u64) -> u64 {
    let default = self.pawn_default(at_mask);

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
    let advance = if_mask(
      self.white_turn_mask,
      at_mask.move_up_mask(2),
      at_mask.move_down_mask(2),
    );

    advance & self.empty & can_advance_mask & can_default_mask
  }

  fn pawn_capturing(&self, at_mask: u64) -> u64 {
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

    (enemy_to_left | enemy_to_right) & (self.enemy | self.en_passant_mask)
  }

  fn knight(&self, at_mask: u64) -> u64 {
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

    self.offset(at_mask, offsets)
  }

  fn bishop(&self, at_mask: u64) -> u64 {
    let mut moves = 0;

    moves |= self.iterative(at_mask, -1, -1);
    moves |= self.iterative(at_mask, 1, -1);
    moves |= self.iterative(at_mask, -1, 1);
    moves |= self.iterative(at_mask, 1, 1);

    moves
  }

  fn rook(&self, at_mask: u64) -> u64 {
    let mut moves = 0;

    moves |= self.iterative(at_mask, -1, 0);
    moves |= self.iterative(at_mask, 0, -1);
    moves |= self.iterative(at_mask, 1, 0);
    moves |= self.iterative(at_mask, 0, 1);

    moves
  }

  fn queen(&self, at_mask: u64) -> u64 {
    self.bishop(at_mask) | self.rook(at_mask)
  }

  fn king_default(&self, at_mask: u64) -> u64 {
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

    self.offset(at_mask, offsets)
  }

  fn offset(&self, at_mask: u64, offsets: Vec<(i32, i32)>) -> u64 {
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

      moves |= new_move & (self.empty | self.enemy);
    }

    moves
  }

  fn iterative(&self, at_mask: u64, dx: i32, dy: i32) -> u64 {
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
      let is_enemy = new_move & self.enemy > 0;
      let is_empty = new_move & self.empty > 0;
      let is_ally = !(is_empty || is_enemy);

      moves |= if_bool(is_enemy || is_empty, new_move, 0);

      if !within_board || is_ally || is_enemy {
        break;
      }
    }

    moves
  }

  fn king_long_castle(&self, at_mask: u64) -> u64 {
    let rooks = self.rooks & self.ally;
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

  fn king_short_castle(&self, at_mask: u64) -> u64 {
    let rooks = self.rooks & self.ally;
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

  fn pin_filter(&mut self, at_mask: u64) -> u64 {
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
    let diag = (self.bishops | self.queens) & self.enemy;
    let not_diag = (self.rooks | self.queens) & self.enemy;
    let ally_king = self.kings & self.ally;

    self.save();
    for (dx, dy, diag_mask) in directions {
      let pinner = if_mask(diag_mask, diag, not_diag);

      self.ally_is_enemy();
      let king_to_pin_path = self.iterative(ally_king, dx, dy);
      let king_hits_pin = king_to_pin_path & at_mask > 0;

      self.load();
      let pin_to_pinner_path = self.iterative(at_mask, dx, dy);
      let pin_hits_pinner = pin_to_pinner_path & pinner > 0;

      let pinned = mask_from_bool(king_hits_pin && pin_hits_pinner);
      pin_path |= pinned & (king_to_pin_path | pin_to_pinner_path) & !at_mask;
    }

    //king not under attack? kinda discovered pin and king attack
    if_bool(pin_path > 0, pin_path, u64::MAX)
  }

  fn check_filter(&self, at_mask: u64) -> u64 {
    let ally_king = self.kings & self.ally;
    let mut filter = 0;
    filter |= self.pawn_capturing(ally_king) & (self.pawns & self.enemy);
    filter |= self.knight(ally_king) & (self.knights & self.enemy);
    filter |= mask_from_bool(at_mask == ally_king);

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

    let diag = self.enemy & (self.bishops | self.queens);
    let not_diag = self.enemy & (self.rooks | self.queens);

    for (dx, dy, diag_mask) in directions {
      let attacker = if_mask(diag_mask, diag, not_diag);

      let king_to_attacker_path = self.iterative(ally_king, dx, dy);
      let attacker_mask = king_to_attacker_path & attacker;
      let king_hits_attacker = mask_from_bool(attacker_mask > 0);

      filter |= king_hits_attacker & (king_to_attacker_path | attacker);
    }

    if_bool(filter > 0, filter, u64::MAX)
  }
}

//other
impl MoveGen {
  fn switch_turn(&mut self) {
    self.white_turn_mask = !self.white_turn_mask;
    mem::swap(&mut self.ally, &mut self.enemy);
  }

  fn ally_is_enemy(&mut self) {
    self.enemy |= self.ally;
    self.ally = 0;
  }

  fn empty_is_enemy(&mut self) {
    self.enemy |= self.empty;
    self.empty = 0;
  }

  fn skip_enemy_king(&mut self) {
    self.enemy &= !(self.kings);
    self.empty = !(self.ally | self.enemy)
  }

  fn save(&mut self) {
    self.saved_white_turn_mask = self.white_turn_mask;
    self.saved_ally = self.ally;
    self.saved_enemy = self.enemy;
    self.saved_empty = self.empty;
  }

  fn load(&mut self) {
    self.white_turn_mask = self.saved_white_turn_mask;
    self.ally = self.saved_ally;
    self.enemy = self.saved_enemy;
    self.empty = self.saved_empty;
  }

  fn reset(&mut self) {
    self.white_turn_mask = self.default_white_turn_mask;
    self.ally = self.default_ally;
    self.enemy = self.default_enemy;
    self.empty = self.default_empty;
  }

  pub(super) fn status(&self) -> u64 {
    let checked = self.king_danger & (self.kings & self.ally) > 0;
    // no pin / check filters for now
    let moves = self.pawn_default(self.pawns & self.ally)
      | self.pawn_advance(self.pawns & self.ally)
      | self.pawn_capturing(self.pawns & self.ally)
      | self.knight(self.knights & self.ally)
      | self.bishop(self.bishops & self.ally)
      | self.rook(self.rooks & self.ally)
      | self.queen(self.queens & self.ally)
      | self.king_default(self.kings & self.ally)
      | self.king_short_castle(self.kings & self.ally)
      | self.king_long_castle(self.kings & self.ally);

    let no_moves = moves == 0;
    let winner = if_mask(self.white_turn_mask, BLACK_WON, WHITE_WON);

    let checkmate = mask_from_bool(checked && no_moves);
    let stalemate = mask_from_bool(!checked && no_moves);

    let king_vs_king = self.ally | self.enemy == self.kings;

    let ally_knight_count = (self.ally & self.knights).count_ones();
    let enemy_knight_count = (self.enemy & self.bishops).count_ones();
    let ally_bishop_count = (self.ally & self.bishops).count_ones();
    let enemy_bishop_count = (self.enemy & self.bishops).count_ones();

    let king_minor_vs_king = (self.ally | self.enemy) == (self.kings | self.knights | self.bishops)
      && (ally_bishop_count + ally_knight_count + enemy_bishop_count + enemy_knight_count == 1);

    let same_square_color = (self.bishops & self.ally).trailing_zeros() % 2
      == (self.bishops & self.enemy).trailing_zeros() % 2;

    let king_bishop_vs_king_bishop = (self.ally | self.enemy) == (self.kings | self.bishops)
      && (ally_bishop_count == 1 && enemy_bishop_count == 1)
      && same_square_color;

    let insufficient_material =
      mask_from_bool(king_vs_king || king_minor_vs_king | king_bishop_vs_king_bishop);

    (checkmate & winner) | (insufficient_material | stalemate & DRAW)
  }
}

#[cfg(test)]
mod tests {
  mod pawn {
    use crate::board::{Board, move_gen::MoveGen};

    #[test]
    fn default() {
      let board = Board::from_fen("k7/p7/8/8/8/8/P7/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      assert_eq!(
        movegen.pawn_default(0x00_80_00_00_00_00_00_00),
        0x00_00_80_00_00_00_00_00
      );
      movegen.switch_turn();
      assert_eq!(
        movegen.pawn_default(0x00_00_00_00_00_00_80_00),
        0x00_00_00_00_00_80_00_00
      );
    }

    #[test]
    fn default_blocked() {
      let board = Board::from_fen("k7/p7/P7/8/8/p7/P7/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      assert_eq!(movegen.pawn_default(0x00_80_00_00_00_00_00_00), 0);
      movegen.switch_turn();
      assert_eq!(movegen.pawn_default(0x00_00_00_00_00_00_80_00), 0);
    }

    #[test]
    fn advance() {
      let board = Board::from_fen("k7/p7/8/8/8/8/P7/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      assert_eq!(
        movegen.pawn_advance(0x00_80_00_00_00_00_00_00),
        0x00_00_00_80_00_00_00_00
      );
      movegen.switch_turn();
      assert_eq!(
        movegen.pawn_advance(0x00_00_00_00_00_00_80_00),
        0x00_00_00_00_80_00_00_00
      );
    }

    #[test]
    fn advance_blocked() {
      let board = Board::from_fen("k7/p7/P7/8/8/p7/P7/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      assert_eq!(movegen.pawn_advance(0x00_80_00_00_00_00_00_00), 0);
      movegen.switch_turn();
      assert_eq!(movegen.pawn_advance(0x00_00_00_00_00_00_80_00), 0);
    }

    #[test]
    fn advance_already_moved() {
      let board = Board::from_fen("k7/8/p7/8/8/7P/8/7K w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      assert_eq!(movegen.pawn_advance(0x00_80_00_00_00_00_00_00), 0);
      movegen.switch_turn();
      assert_eq!(movegen.pawn_advance(0x00_00_00_00_00_00_80_00), 0);
    }

    #[test]
    fn capturing() {
      let board = Board::from_fen("k7/1p6/P1P5/8/8/p1p5/1P6/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      assert_eq!(
        movegen.pawn_capturing(0x00_40_00_00_00_00_00_00),
        0x00_00_A0_00_00_00_00_00
      );
      movegen.switch_turn();
      assert_eq!(
        movegen.pawn_capturing(0x00_00_00_00_00_00_40_00),
        0x00_00_00_00_00_A0_00_00
      );
    }

    #[test]
    fn pinned_vert() {
      let board = Board::from_fen("r6k/7p/6P1/8/8/1p6/P7/K6R w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let white_pawn = 0x00_80_00_00_00_00_00_00;
      let filter = movegen.pin_filter(white_pawn);

      assert_eq!(filter, 0x00_00_80_80_80_80_80_80);
      assert_eq!(
        movegen.pawn_default(white_pawn) & filter,
        0x00_00_80_00_00_00_00_00
      );
      assert_eq!(
        movegen.pawn_advance(white_pawn) & filter,
        0x00_00_00_80_00_00_00_00
      );
      assert_eq!(movegen.pawn_capturing(white_pawn) & filter, 0);

      movegen.switch_turn();

      let black_pawn = 0x00_00_00_00_00_00_01_00;
      let filter = movegen.pin_filter(black_pawn);

      assert_eq!(filter, 0x01_01_01_01_01_01_00_00);
      assert_eq!(
        movegen.pawn_default(black_pawn) & filter,
        0x00_00_00_00_00_01_00_00
      );
      assert_eq!(
        movegen.pawn_advance(black_pawn) & filter,
        0x00_00_00_00_01_00_00_00
      );
      assert_eq!(movegen.pawn_capturing(black_pawn) & filter, 0);
    }

    #[test]
    fn pinned_hor() {
      let board = Board::from_fen("kp5R/P7/8/8/8/8/p7/KP5r w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let white_pawn = 0x40_00_00_00_00_00_00_00;
      let filter = movegen.pin_filter(white_pawn);

      assert_eq!(filter, 0x3F_00_00_00_00_00_00_00);
      assert_eq!(movegen.pawn_default(white_pawn) & filter, 0);
      assert_eq!(movegen.pawn_advance(white_pawn) & filter, 0);
      assert_eq!(movegen.pawn_capturing(white_pawn) & filter, 0);

      movegen.switch_turn();

      let black_pawn = 0x00_00_00_00_00_00_00_40;
      let filter = movegen.pin_filter(black_pawn);

      assert_eq!(filter, 0x00_00_00_00_00_00_00_3F);
      assert_eq!(movegen.pawn_default(black_pawn) & filter, 0);
      assert_eq!(movegen.pawn_advance(black_pawn) & filter, 0);
      assert_eq!(movegen.pawn_capturing(black_pawn) & filter, 0);
    }

    #[test]
    fn pinned_diag() {
      let board = Board::from_fen("k6b/1p6/P7/8/8/p7/1P6/K6B w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let white_pawn = 0x00_40_00_00_00_00_00_00;
      let filter = movegen.pin_filter(white_pawn);

      assert_eq!(filter, 0x00_00_20_10_08_04_02_01);
      assert_eq!(movegen.pawn_default(white_pawn) & filter, 0);
      assert_eq!(movegen.pawn_advance(white_pawn) & filter, 0);
      assert_eq!(movegen.pawn_capturing(white_pawn) & filter, 0);

      movegen.switch_turn();

      let black_pawn = 0x00_00_00_00_00_00_40_00;
      let filter = movegen.pin_filter(black_pawn);

      assert_eq!(filter, 0x01_02_04_08_10_20_00_00);
      assert_eq!(movegen.pawn_default(black_pawn) & filter, 0);
      assert_eq!(movegen.pawn_advance(black_pawn) & filter, 0);
      assert_eq!(movegen.pawn_capturing(black_pawn) & filter, 0);
    }
  }

  mod knight {
    use crate::board::{Board, move_gen::MoveGen};

    #[test]
    fn default() {
      let board = Board::from_fen("k7/8/8/3n4/4N3/8/8/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      assert_eq!(
        movegen.knight(0x00_00_00_10_00_00_00_00),
        0x00_28_44_00_44_28_00_00
      );
      movegen.switch_turn();
      assert_eq!(
        movegen.knight(0x00_00_00_00_10_00_00_00),
        0x00_00_28_44_00_44_28_00
      );
    }

    #[test]
    fn default_on_edge() {
      let board = Board::from_fen("N6k/8/8/8/8/8/8/K6n w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let ally_knights = movegen.knights & movegen.ally;
      assert_eq!(movegen.knight(ally_knights), 0x00_00_00_00_00_40_20_00);
      movegen.switch_turn();
      let ally_knights = movegen.knights & movegen.ally;
      assert_eq!(movegen.knight(ally_knights), 0x00_04_02_00_00_00_00_00);
    }

    #[test]
    fn pinned_vert() {
      let board = Board::from_fen("r6k/7n/8/6P1/1p6/8/N7/K6R w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let ally_knight = movegen.knights & movegen.ally;
      let filter = movegen.pin_filter(ally_knight);

      assert_eq!(filter, 0x00_00_80_80_80_80_80_80);
      assert_eq!(movegen.knight(ally_knight) & filter, 0);

      movegen.switch_turn();

      let ally_knight = movegen.knights & movegen.ally;
      let filter = movegen.pin_filter(ally_knight);

      assert_eq!(filter, 0x01_01_01_01_01_01_00_00);
      assert_eq!(movegen.knight(ally_knight) & filter, 0);
    }

    #[test]
    fn pinned_hor() {
      let board = Board::from_fen("kn5R/8/P7/8/8/p7/8/KN5r w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let ally_knight = movegen.knights & movegen.ally;
      let filter = movegen.pin_filter(ally_knight);

      assert_eq!(filter, 0x3F_00_00_00_00_00_00_00);
      assert_eq!(movegen.knight(ally_knight) & filter, 0);

      movegen.switch_turn();

      let ally_knight = movegen.knights & movegen.ally;
      let filter = movegen.pin_filter(ally_knight);

      assert_eq!(filter, 0x00_00_00_00_00_00_00_3F);
      assert_eq!(movegen.knight(ally_knight) & filter, 0);
    }

    #[test]
    fn pinned_diag() {
      let board = Board::from_fen("k6b/1n6/8/P7/p7/8/1N6/K6B w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let ally_knight = movegen.knights & movegen.ally;
      let filter = movegen.pin_filter(ally_knight);

      assert_eq!(filter, 0x00_00_20_10_08_04_02_01);
      assert_eq!(movegen.knight(ally_knight) & filter, 0);

      movegen.switch_turn();

      let ally_knight = movegen.knights & movegen.ally;
      let filter = movegen.pin_filter(ally_knight);

      assert_eq!(filter, 0x01_02_04_08_10_20_00_00);
      assert_eq!(movegen.knight(ally_knight) & filter, 0);
    }
  }

  mod bishop {
    use crate::board::{Board, move_gen::MoveGen};

    #[test]
    fn default() {
      let board = Board::from_fen("k7/8/8/3b4/3B4/8/8/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let ally_bishop = movegen.bishops & movegen.ally;
      assert_eq!(movegen.bishop(ally_bishop), 0x02_44_28_00_28_44_82_01);

      movegen.switch_turn();
      let ally_bishop = movegen.bishops & movegen.ally;
      assert_eq!(movegen.bishop(ally_bishop), 0x01_82_44_28_00_28_44_02);
    }

    #[test]
    fn default_on_edge() {
      let board = Board::from_fen("kb6/8/8/8/8/8/8/KB6 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let ally_bishop = movegen.bishops & movegen.ally;
      assert_eq!(movegen.bishop(ally_bishop), 0x00_A0_10_08_04_02_01_00);
      movegen.switch_turn();
      let ally_bishop = movegen.bishops & movegen.ally;
      assert_eq!(movegen.bishop(ally_bishop), 0x00_01_02_04_08_10_A0_00);
    }

    #[test]
    fn pinned_vert() {
      let board = Board::from_fen("k6K/b6B/8/8/8/8/8/R6r w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let ally_bishop = movegen.bishops & movegen.ally;
      let filter = movegen.pin_filter(ally_bishop);

      assert_eq!(filter, 0x01_01_01_01_01_01_00_00);
      assert_eq!(movegen.bishop(ally_bishop) & filter, 0);

      movegen.switch_turn();

      let ally_bishop = movegen.bishops & movegen.ally;
      let filter = movegen.pin_filter(ally_bishop);

      assert_eq!(filter, 0x80_80_80_80_80_80_00_00);
      assert_eq!(movegen.bishop(ally_bishop) & filter, 0);
    }

    #[test]
    fn pinned_hor() {
      let board = Board::from_fen("kb5R/8/8/8/8/8/8/KB5r w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let ally_bishop = movegen.bishops & movegen.ally;
      let filter = movegen.pin_filter(ally_bishop);

      assert_eq!(filter, 0x3F_00_00_00_00_00_00_00);
      assert_eq!(movegen.bishop(ally_bishop) & filter, 0);

      movegen.switch_turn();

      let ally_bishop = movegen.bishops & movegen.ally;
      let filter = movegen.pin_filter(ally_bishop);

      assert_eq!(filter, 0x00_00_00_00_00_00_00_3F);
      assert_eq!(movegen.bishop(ally_bishop) & filter, 0);
    }

    #[test]
    fn pinned_diag() {
      let board = Board::from_fen("k6q/1b6/8/8/8/8/1B6/K6Q w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let ally_bishop = movegen.bishops & movegen.ally;
      let filter = movegen.pin_filter(ally_bishop);

      assert_eq!(filter, 0x00_00_20_10_08_04_02_01);
      assert_eq!(
        movegen.bishop(ally_bishop) & filter,
        0x00_00_20_10_08_04_02_01
      );

      movegen.switch_turn();

      let ally_bishop = movegen.bishops & movegen.ally;
      let filter = movegen.pin_filter(ally_bishop);

      assert_eq!(filter, 0x01_02_04_08_10_20_00_00);
      assert_eq!(
        movegen.bishop(ally_bishop) & filter,
        0x01_02_04_08_10_20_00_00
      );
    }
  }

  mod rook {
    use crate::board::{Board, move_gen::MoveGen};

    #[test]
    fn default() {
      let board = Board::from_fen("k7/8/8/3r4/4R3/8/8/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let ally_rook = movegen.rooks & movegen.ally;
      assert_eq!(movegen.rook(ally_rook), 0x08_08_08_F7_08_08_08_08);

      movegen.switch_turn();
      let ally_rook = movegen.rooks & movegen.ally;
      assert_eq!(movegen.rook(ally_rook), 0x10_10_10_10_EF_10_10_10);
    }

    #[test]
    fn default_on_edge() {
      let board = Board::from_fen("k6r/8/8/8/8/8/8/K6R w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let ally_rook = movegen.rooks & movegen.ally;
      assert_eq!(movegen.rook(ally_rook), 0x7E_01_01_01_01_01_01_01);
      movegen.switch_turn();
      let ally_rook = movegen.rooks & movegen.ally;
      assert_eq!(movegen.rook(ally_rook), 0x01_01_01_01_01_01_01_7E);
    }

    #[test]
    fn pinned_vert() {
      let board = Board::from_fen("k6K/r6R/8/8/8/8/8/Q6q w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let ally_rook = movegen.rooks & movegen.ally;
      let filter = movegen.pin_filter(ally_rook);

      assert_eq!(filter, 0x01_01_01_01_01_01_00_00);
      assert_eq!(movegen.rook(ally_rook) & filter, 0x01_01_01_01_01_01_00_00);

      movegen.switch_turn();

      let ally_rook = movegen.rooks & movegen.ally;
      let filter = movegen.pin_filter(ally_rook);

      assert_eq!(filter, 0x80_80_80_80_80_80_00_00);
      assert_eq!(movegen.rook(ally_rook) & filter, 0x80_80_80_80_80_80_00_00);
    }

    #[test]
    fn pinned_hor() {
      let board = Board::from_fen("kr5Q/8/8/8/8/8/8/KR5q w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let ally_rook = movegen.rooks & movegen.ally;
      let filter = movegen.pin_filter(ally_rook);

      assert_eq!(filter, 0x3F_00_00_00_00_00_00_00);
      assert_eq!(movegen.rook(ally_rook) & filter, 0x3F_00_00_00_00_00_00_00);

      movegen.switch_turn();

      let ally_rook = movegen.rooks & movegen.ally;
      let filter = movegen.pin_filter(ally_rook);

      assert_eq!(filter, 0x00_00_00_00_00_00_00_3F);
      assert_eq!(movegen.rook(ally_rook) & filter, 0x00_00_00_00_00_00_00_3F);
    }

    #[test]
    fn pinned_diag() {
      let board = Board::from_fen("k6b/1r6/8/8/8/8/1R6/K6B w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let ally_rook = movegen.rooks & movegen.ally;
      let filter = movegen.pin_filter(ally_rook);

      assert_eq!(filter, 0x00_00_20_10_08_04_02_01);
      assert_eq!(movegen.rook(ally_rook) & filter, 0);

      movegen.switch_turn();

      let ally_rook = movegen.rooks & movegen.ally;
      let filter = movegen.pin_filter(ally_rook);

      assert_eq!(filter, 0x01_02_04_08_10_20_00_00);
      assert_eq!(movegen.rook(ally_rook) & filter, 0);
    }
  }
}
