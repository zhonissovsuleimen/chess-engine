use super::{
  Board, board_movement_trait::BoardMovement, cached_piece_moves::CachedPieceMoves, status::*,
  util_fns::*,
};

pub struct MoveGen {
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

  en_passant_mask: u64,
  white_short_castle: bool,
  white_long_castle: bool,
  black_short_castle: bool,
  black_long_castle: bool,
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

    let movegen = MoveGen {
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

      en_passant_mask: board.en_passant_mask,
      white_short_castle: board.white_short_castle,
      white_long_castle: board.white_long_castle,
      black_short_castle: board.black_short_castle,
      black_long_castle: board.black_short_castle,
    };

    movegen
  }

  pub(super) fn cached(board: &Board, at_mask: u64) -> CachedPieceMoves {
    let mut movegen = MoveGen::default(board);
    let pin = movegen.pin_filter(at_mask);
    let check = movegen.check_filter();
    let filter = pin & check;

    let mut moves = CachedPieceMoves {
      from_mask: at_mask,
      pawn_default: movegen.pawn_default(at_mask & movegen.pawns & movegen.ally) & filter,
      pawn_advance: movegen.pawn_advance(at_mask & movegen.pawns & movegen.ally) & filter,
      pawn_capture: movegen.pawn_capture(at_mask & movegen.pawns & movegen.ally) & filter,
      knight: movegen.knight(at_mask & movegen.knights & movegen.ally) & filter,
      bishop: movegen.bishop(at_mask & movegen.bishops & movegen.ally) & filter,
      rook: movegen.rook(at_mask & movegen.rooks & movegen.ally) & filter,
      queen: movegen.queen(at_mask & movegen.queens & movegen.ally) & filter,
      king_default: movegen.king_default(at_mask & movegen.kings & movegen.ally)
        & !movegen.king_danger(),
      king_short_castle: movegen.king_short_castle(),
      king_long_castle: movegen.king_long_castle(),
      capturing: 0,
    };
    moves.capturing = moves.all() & movegen.enemy;
    moves
  }
}

//move gen
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
    let default = if_mask(
      self.white_turn_mask,
      at_mask.move_up_mask(1),
      at_mask.move_down_mask(1),
    );
    let advance = if_mask(
      self.white_turn_mask,
      at_mask.move_up_mask(2),
      at_mask.move_down_mask(2),
    );
    let advance_mask = if_mask(
      self.white_turn_mask,
      0x00_FF_00_00_00_00_00_00,
      0x00_00_00_00_00_00_FF_00,
    );

    let can_default = default & self.empty > 0;
    let can_advance = advance & self.empty > 0 && at_mask & advance_mask > 0;

    advance & mask_from_bool(can_default && can_advance)
  }

  fn pawn_capture(&self, at_mask: u64) -> u64 {
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

    let pos_dx = mask_from_bool(dx > 0);
    let pos_dy = mask_from_bool(dy > 0);

    for i in 1..=7 {
      let mut new_move = if_mask(
        pos_dx,
        at_mask.move_right_mask(i * dx.unsigned_abs()),
        at_mask.move_left_mask(i * dx.unsigned_abs()),
      );
      new_move = if_mask(
        pos_dy,
        new_move.move_up_mask(i * dy.unsigned_abs()),
        new_move.move_down_mask(i * dy.unsigned_abs()),
      );

      let within_board = new_move > 0;
      let is_enemy = new_move & self.enemy > 0;
      let is_empty = new_move & self.empty > 0;

      moves |= new_move & mask_from_bool(is_enemy || is_empty);

      if !within_board || !is_empty {
        break;
      }
    }

    moves
  }

  fn king_long_castle(&mut self) -> u64 {
    let king_danger = self.king_danger();
    let rook_u64 = if_mask(
      self.white_turn_mask,
      0x80_00_00_00_00_00_00_00,
      0x00_00_00_00_00_00_00_80,
    );
    let king_u64 = if_mask(
      self.white_turn_mask,
      0x08_00_00_00_00_00_00_00,
      0x00_00_00_00_00_00_00_08,
    );
    let empty_u64 = if_mask(
      self.white_turn_mask,
      0x70_00_00_00_00_00_00_00,
      0x00_00_00_00_00_00_00_70,
    );
    let safe_u64 = if_mask(
      self.white_turn_mask,
      0x38_00_00_00_00_00_00_00,
      0x00_00_00_00_00_00_00_38,
    );
    let new_pos_u64 = if_mask(
      self.white_turn_mask,
      0x20_00_00_00_00_00_00_00,
      0x00_00_00_00_00_00_00_20,
    );

    let king = self.ally & self.kings == king_u64;
    let rook = rook_u64 & self.rooks & self.ally > 0;
    let empty = self.empty & empty_u64 == empty_u64;
    let safe = !king_danger & safe_u64 == safe_u64;

    let white_turn = self.white_turn_mask == 0xFF_FF_FF_FF_FF_FF_FF_FF;
    let rights = (white_turn && self.white_long_castle) || (!white_turn && self.black_long_castle);

    new_pos_u64 & mask_from_bool(rights && king && rook && empty && safe)
  }

  fn king_short_castle(&mut self) -> u64 {
    let king_danger = self.king_danger();
    let rook_u64 = if_mask(
      self.white_turn_mask,
      0x01_00_00_00_00_00_00_00,
      0x00_00_00_00_00_00_00_01,
    );
    let king_u64 = if_mask(
      self.white_turn_mask,
      0x08_00_00_00_00_00_00_00,
      0x00_00_00_00_00_00_00_08,
    );
    let empty_u64 = if_mask(
      self.white_turn_mask,
      0x06_00_00_00_00_00_00_00,
      0x00_00_00_00_00_00_00_06,
    );
    let safe_u64 = if_mask(
      self.white_turn_mask,
      0x0E_00_00_00_00_00_00_00,
      0x00_00_00_00_00_00_00_0E,
    );
    let new_pos_u64 = if_mask(
      self.white_turn_mask,
      0x02_00_00_00_00_00_00_00,
      0x00_00_00_00_00_00_00_02,
    );

    let king = self.ally & self.kings == king_u64;
    let rook = rook_u64 & self.rooks & self.ally > 0;
    let empty = self.empty & empty_u64 == empty_u64;
    let safe = !king_danger & safe_u64 == safe_u64;

    let white_turn = self.white_turn_mask == 0xFF_FF_FF_FF_FF_FF_FF_FF;
    let rights = (white_turn && self.white_short_castle) || (!white_turn && self.black_short_castle);

    new_pos_u64 & mask_from_bool(rights && king && rook && empty && safe)
  }
}

//other
impl MoveGen {
  fn pin_filter(&mut self, at_mask: u64) -> u64 {
    let diag = (self.bishops | self.queens) & self.enemy;
    let not_diag = (self.rooks | self.queens) & self.enemy;

    let directions = [
      (-1, 1, diag),
      (0, 1, not_diag),
      (1, 1, diag),
      (-1, 0, not_diag),
      (1, 0, not_diag),
      (-1, -1, diag),
      (0, -1, not_diag),
      (1, -1, diag),
    ];

    let mut pin_path = 0;
    let ally_king = self.kings & self.ally;

    self.save();
    for (dx, dy, diag) in directions {
      self.remove_piece(at_mask);
      let king_to_pinner_path = self.iterative(ally_king, dx, dy);
      let king_hits_pinner = king_to_pinner_path & diag > 0;
      self.load();
      let path_contains_pin = king_to_pinner_path & at_mask > 0;

      let pinned = mask_from_bool(king_hits_pinner & path_contains_pin);
      pin_path |= pinned & king_to_pinner_path & !at_mask;
    }

    mask_from_bool(pin_path == 0) | pin_path
  }

  fn check_filter(&self) -> u64 {
    let ally_king = self.kings & self.ally;
    let mut filter = 0;
    filter |= self.pawn_capture(ally_king) & (self.pawns & self.enemy);
    filter |= self.knight(ally_king) & (self.knights & self.enemy);

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
      let king_hits_attacker = mask_from_bool(king_to_attacker_path & attacker > 0);

      filter |= king_hits_attacker & king_to_attacker_path;
    }

    if_bool(filter > 0, filter, u64::MAX)
  }

  fn king_danger(&mut self) -> u64 {
    let mut king_danger = 0;
    self.switch_turn();
    self.save();

    self.empty_is_enemy();
    king_danger |= self.pawn_capture(self.pawns & self.ally);
    king_danger |= self.knight(self.knights & self.ally);
    king_danger |= self.king_default(self.kings & self.ally);

    self.load();
    self.remove_piece(self.kings & self.enemy);
    king_danger |= self.bishop(self.bishops & self.ally);
    king_danger |= self.rook(self.rooks & self.ally);
    king_danger |= self.queen(self.queens & self.ally);

    self.load();
    self.switch_turn();
    king_danger
  }

  pub fn get_status(&mut self) -> u64 {
    let filter = self.check_filter();
    let checked = self.king_danger() & (self.kings & self.ally) > 0;
    let danger = self.king_danger();
    let mut moves = self.pawn_default(self.pawns & self.ally) 
      | self.pawn_advance(self.pawns & self.ally) 
      | self.pawn_capture(self.pawns & self.ally) 
      | self.knight(self.knights & self.ally) 
      | self.bishop((self.queens | self.bishops) & self.ally) 
      | self.rook((self.queens | self.rooks) & self.ally) 
      | self.king_short_castle() 
      | self.king_long_castle();

    moves = moves & filter | (self.king_default(self.kings & self.ally) & !danger);

    let no_moves = moves == 0;
    let winner = if_mask(self.white_turn_mask, BLACK_WON, WHITE_WON);

    let checkmate = mask_from_bool(checked && no_moves);
    let stalemate = mask_from_bool(!checked && no_moves);

    let king_vs_king = self.ally | self.enemy == self.kings;

    let knight_count = self.knights.count_ones();
    let ally_bishop_count = (self.ally & self.bishops).count_ones();
    let enemy_bishop_count = (self.enemy & self.bishops).count_ones();

    let king_minor_vs_king = (self.ally | self.enemy) == (self.kings | self.knights | self.bishops)
      && (knight_count + ally_bishop_count + enemy_bishop_count == 1);

    let black_squares = 0xAA_55_AA_55_AA_55_AA_55_u64;
    let same_square_color = (self.bishops & black_squares).count_ones() > 0;

    let king_bishop_vs_king_bishop = (self.ally | self.enemy) == (self.kings | self.bishops)
      && (ally_bishop_count == 1 && enemy_bishop_count == 1)
      && same_square_color;

    let insufficient_material =
      mask_from_bool(king_vs_king || king_minor_vs_king | king_bishop_vs_king_bishop);

    (checkmate & winner) | ((insufficient_material | stalemate) & DRAW)
  }
}

//checkpoints / modifiers
impl MoveGen {
  fn switch_turn(&mut self) {
    self.white_turn_mask = !self.white_turn_mask;
    std::mem::swap(&mut self.ally, &mut self.enemy);
  }

  fn empty_is_enemy(&mut self) {
    self.enemy |= self.empty;
    self.empty = 0;
  }

  fn remove_piece(&mut self, at_mask: u64) {
    self.ally &= !(at_mask);
    self.enemy &= !(at_mask);
    self.empty |= at_mask;
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
}

#[cfg(test)]
mod tests {
  mod pawn {
    use crate::board::{
      Board,
      move_gen::MoveGen,
      move_input::{BISHOP, KNIGHT, MoveInput, NONE, QUEEN, ROOK},
    };

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

      let pawn = movegen.pawns & movegen.ally;
      assert_eq!(movegen.pawn_advance(pawn), 0);
      movegen.switch_turn();
      let pawn = movegen.pawns & movegen.ally;
      assert_eq!(movegen.pawn_advance(pawn), 0);
    }

    #[test]
    fn capture() {
      let board = Board::from_fen("k7/1p6/P1P5/8/8/p1p5/1P6/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      assert_eq!(
        movegen.pawn_capture(0x00_40_00_00_00_00_00_00),
        0x00_00_A0_00_00_00_00_00
      );
      movegen.switch_turn();
      assert_eq!(
        movegen.pawn_capture(0x00_00_00_00_00_00_40_00),
        0x00_00_00_00_00_A0_00_00
      );
    }

    #[test]
    fn en_passant() {
      let mut board = Board::from_fen("k7/p7/8/1P6/1p6/8/P7/K7 w - - 0 1");
      board.move_piece(MoveInput::from_id(55, 39));
      let movegen = MoveGen::default(&board);
      assert_eq!(
        movegen.pawn_capture(0x00_00_00_40_00_00_00_00),
        0x00_00_80_00_00_00_00_00
      );

      let mut board = Board::from_fen("k7/p7/8/1P6/1p6/8/P7/K7 b - - 0 1");
      board.move_piece(MoveInput::from_id(15, 31));
      let movegen = MoveGen::default(&board);
      assert_eq!(
        movegen.pawn_capture(0x00_00_00_00_40_00_00_00),
        0x00_00_00_00_00_80_00_00
      );
    }

    #[test]
    fn promote_to_knight() {
      let mut board = Board::from_fen("8/PK6/8/8/8/8/pk6/8 w - - 0 1");
      let mut movegen = MoveGen::default(&board);
      assert_eq!(
        movegen.pawn_default(0x00_00_00_00_00_00_80_00),
        0x00_00_00_00_00_00_00_80
      );

      movegen.switch_turn();
      assert_eq!(
        movegen.pawn_default(0x00_80_00_00_00_00_00_00),
        0x80_00_00_00_00_00_00_00
      );

      board.move_piece(MoveInput::with_promotion(
        0x00_00_00_00_00_00_80_00,
        0x00_00_00_00_00_00_00_80,
        KNIGHT,
      ));

      board.move_piece(MoveInput::with_promotion(
        0x00_80_00_00_00_00_00_00,
        0x80_00_00_00_00_00_00_00,
        KNIGHT,
      ));

      let movegen = MoveGen::default(&board);
      assert_eq!(movegen.pawns, 0);
      assert_eq!(movegen.knights & movegen.ally, 0x00_00_00_00_00_00_00_80);
      assert_eq!(movegen.knights & movegen.enemy, 0x80_00_00_00_00_00_00_00);
    }

    #[test]
    fn promote_to_bishop() {
      let mut board = Board::from_fen("8/PK6/8/8/8/8/pk6/8 w - - 0 1");
      let mut movegen = MoveGen::default(&board);
      assert_eq!(
        movegen.pawn_default(0x00_00_00_00_00_00_80_00),
        0x00_00_00_00_00_00_00_80
      );

      movegen.switch_turn();
      assert_eq!(
        movegen.pawn_default(0x00_80_00_00_00_00_00_00),
        0x80_00_00_00_00_00_00_00
      );

      board.move_piece(MoveInput::with_promotion(
        0x00_00_00_00_00_00_80_00,
        0x00_00_00_00_00_00_00_80,
        BISHOP,
      ));

      board.move_piece(MoveInput::with_promotion(
        0x00_80_00_00_00_00_00_00,
        0x80_00_00_00_00_00_00_00,
        BISHOP,
      ));

      let movegen = MoveGen::default(&board);
      assert_eq!(movegen.pawns, 0);
      assert_eq!(movegen.bishops & movegen.ally, 0x00_00_00_00_00_00_00_80);
      assert_eq!(movegen.bishops & movegen.enemy, 0x80_00_00_00_00_00_00_00);
    }

    #[test]
    fn promote_to_rook() {
      let mut board = Board::from_fen("8/PK6/8/8/8/8/pk6/8 w - - 0 1");
      let mut movegen = MoveGen::default(&board);
      assert_eq!(
        movegen.pawn_default(0x00_00_00_00_00_00_80_00),
        0x00_00_00_00_00_00_00_80
      );

      movegen.switch_turn();
      assert_eq!(
        movegen.pawn_default(0x00_80_00_00_00_00_00_00),
        0x80_00_00_00_00_00_00_00
      );

      board.move_piece(MoveInput::with_promotion(
        0x00_00_00_00_00_00_80_00,
        0x00_00_00_00_00_00_00_80,
        ROOK,
      ));

      board.move_piece(MoveInput::with_promotion(
        0x00_80_00_00_00_00_00_00,
        0x80_00_00_00_00_00_00_00,
        ROOK,
      ));

      let movegen = MoveGen::default(&board);
      assert_eq!(movegen.pawns, 0);
      assert_eq!(movegen.rooks & movegen.ally, 0x00_00_00_00_00_00_00_80);
      assert_eq!(movegen.rooks & movegen.enemy, 0x80_00_00_00_00_00_00_00);
    }

    #[test]
    fn promote_to_queen() {
      let mut board = Board::from_fen("8/PK6/8/8/8/8/pk6/8 w - - 0 1");
      let mut movegen = MoveGen::default(&board);
      assert_eq!(
        movegen.pawn_default(0x00_00_00_00_00_00_80_00),
        0x00_00_00_00_00_00_00_80
      );

      movegen.switch_turn();
      assert_eq!(
        movegen.pawn_default(0x00_80_00_00_00_00_00_00),
        0x80_00_00_00_00_00_00_00
      );

      board.move_piece(MoveInput::with_promotion(
        0x00_00_00_00_00_00_80_00,
        0x00_00_00_00_00_00_00_80,
        QUEEN,
      ));

      board.move_piece(MoveInput::with_promotion(
        0x00_80_00_00_00_00_00_00,
        0x80_00_00_00_00_00_00_00,
        QUEEN,
      ));

      let movegen = MoveGen::default(&board);
      assert_eq!(movegen.pawns, 0);
      assert_eq!(movegen.queens & movegen.ally, 0x00_00_00_00_00_00_00_80);
      assert_eq!(movegen.queens & movegen.enemy, 0x80_00_00_00_00_00_00_00);
    }

    #[test]
    fn promote_no_choice() {
      let mut board = Board::from_fen("8/PK6/8/8/8/8/pk6/8 w - - 0 1");
      let movegen = MoveGen::default(&board);
      assert_eq!(
        movegen.pawn_default(0x00_00_00_00_00_00_80_00),
        0x00_00_00_00_00_00_00_80
      );
      board.move_piece(MoveInput::with_promotion(
        0x00_00_00_00_00_00_80_00,
        0x00_00_00_00_00_00_00_80,
        NONE,
      ));
      let movegen = MoveGen::default(&board);
      assert_eq!(movegen.pawns & movegen.ally, 0x00_00_00_00_00_00_80_00);
      assert_eq!(movegen.knights, 0);
      assert_eq!(movegen.bishops, 0);
      assert_eq!(movegen.rooks, 0);
      assert_eq!(movegen.queens, 0);

      let mut board = Board::from_fen("8/PK6/8/8/8/8/pk6/8 b - - 0 1");
      let movegen = MoveGen::default(&board);
      assert_eq!(
        movegen.pawn_default(0x00_80_00_00_00_00_00_00),
        0x80_00_00_00_00_00_00_00
      );
      board.move_piece(MoveInput::with_promotion(
        0x00_80_00_00_00_00_00_00,
        0x80_00_00_00_00_00_00_00,
        NONE,
      ));
      let movegen = MoveGen::default(&board);
      assert_eq!(movegen.pawns & movegen.ally, 0x00_80_00_00_00_00_00_00);
      assert_eq!(movegen.knights, 0);
      assert_eq!(movegen.bishops, 0);
      assert_eq!(movegen.rooks, 0);
      assert_eq!(movegen.queens, 0);
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
      assert_eq!(movegen.pawn_capture(white_pawn) & filter, 0);

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
      assert_eq!(movegen.pawn_capture(black_pawn) & filter, 0);
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
      assert_eq!(movegen.pawn_capture(white_pawn) & filter, 0);

      movegen.switch_turn();

      let black_pawn = 0x00_00_00_00_00_00_00_40;
      let filter = movegen.pin_filter(black_pawn);

      assert_eq!(filter, 0x00_00_00_00_00_00_00_3F);
      assert_eq!(movegen.pawn_default(black_pawn) & filter, 0);
      assert_eq!(movegen.pawn_advance(black_pawn) & filter, 0);
      assert_eq!(movegen.pawn_capture(black_pawn) & filter, 0);
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
      assert_eq!(movegen.pawn_capture(white_pawn) & filter, 0);

      movegen.switch_turn();

      let black_pawn = 0x00_00_00_00_00_00_40_00;
      let filter = movegen.pin_filter(black_pawn);

      assert_eq!(filter, 0x01_02_04_08_10_20_00_00);
      assert_eq!(movegen.pawn_default(black_pawn) & filter, 0);
      assert_eq!(movegen.pawn_advance(black_pawn) & filter, 0);
      assert_eq!(movegen.pawn_capture(black_pawn) & filter, 0);
    }

    #[test]
    fn king_checked() {
      let board = Board::from_fen("8/1p6/k6R/8/8/K6r/1P6/8 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let pawn = movegen.pawns & movegen.ally;
      let filter = movegen.check_filter();

      assert_eq!(
        movegen.pawn_default(pawn) & filter,
        0x00_00_40_00_00_00_00_00
      );
      assert_eq!(movegen.pawn_advance(pawn) & filter, 0);

      movegen.switch_turn();

      let pawn = movegen.pawns & movegen.ally;
      let filter = movegen.check_filter();

      assert_eq!(
        movegen.pawn_default(pawn) & filter,
        0x00_00_00_00_00_40_00_00
      );
      assert_eq!(movegen.pawn_advance(pawn) & filter, 0);
    }

    #[test]
    fn pinned_and_king_checked() {
      let board = Board::from_fen("2B5/1p6/k6R/8/8/K6r/1P6/2b5 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let pawn = movegen.pawns & movegen.ally;
      let filter = movegen.pin_filter(pawn) & movegen.check_filter();

      assert_eq!(movegen.pawn_default(pawn) & filter, 0);
      assert_eq!(movegen.pawn_advance(pawn) & filter, 0);

      movegen.switch_turn();

      let pawn = movegen.pawns & movegen.ally;
      let filter = movegen.pin_filter(pawn) & movegen.check_filter();

      assert_eq!(movegen.pawn_default(pawn) & filter, 0);
      assert_eq!(movegen.pawn_advance(pawn) & filter, 0);
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

      let knights = movegen.knights & movegen.ally;
      assert_eq!(movegen.knight(knights), 0x00_00_00_00_00_40_20_00);
      movegen.switch_turn();
      let knights = movegen.knights & movegen.ally;
      assert_eq!(movegen.knight(knights), 0x00_04_02_00_00_00_00_00);
    }

    #[test]
    fn pinned_vert() {
      let board = Board::from_fen("r6k/7n/8/6P1/1p6/8/N7/K6R w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let knight = movegen.knights & movegen.ally;
      let filter = movegen.pin_filter(knight);

      assert_eq!(filter, 0x00_00_80_80_80_80_80_80);
      assert_eq!(movegen.knight(knight) & filter, 0);

      movegen.switch_turn();

      let knight = movegen.knights & movegen.ally;
      let filter = movegen.pin_filter(knight);

      assert_eq!(filter, 0x01_01_01_01_01_01_00_00);
      assert_eq!(movegen.knight(knight) & filter, 0);
    }

    #[test]
    fn pinned_hor() {
      let board = Board::from_fen("kn5R/8/P7/8/8/p7/8/KN5r w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let knight = movegen.knights & movegen.ally;
      let filter = movegen.pin_filter(knight);

      assert_eq!(filter, 0x3F_00_00_00_00_00_00_00);
      assert_eq!(movegen.knight(knight) & filter, 0);

      movegen.switch_turn();

      let knight = movegen.knights & movegen.ally;
      let filter = movegen.pin_filter(knight);

      assert_eq!(filter, 0x00_00_00_00_00_00_00_3F);
      assert_eq!(movegen.knight(knight) & filter, 0);
    }

    #[test]
    fn pinned_diag() {
      let board = Board::from_fen("k6b/1n6/8/P7/p7/8/1N6/K6B w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let knight = movegen.knights & movegen.ally;
      let filter = movegen.pin_filter(knight);

      assert_eq!(filter, 0x00_00_20_10_08_04_02_01);
      assert_eq!(movegen.knight(knight) & filter, 0);

      movegen.switch_turn();

      let knight = movegen.knights & movegen.ally;
      let filter = movegen.pin_filter(knight);

      assert_eq!(filter, 0x01_02_04_08_10_20_00_00);
      assert_eq!(movegen.knight(knight) & filter, 0);
    }

    #[test]
    fn king_checked() {
      let board = Board::from_fen("8/3n4/k6R/8/8/K6r/3N4/8 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let knight = movegen.knights & movegen.ally;
      let filter = movegen.check_filter();

      assert_eq!(movegen.knight(knight) & filter, 0x00_00_44_00_00_00_00_00);

      movegen.switch_turn();

      let knight = movegen.knights & movegen.ally;
      let filter = movegen.check_filter();

      assert_eq!(movegen.knight(knight) & filter, 0x00_00_00_00_00_44_00_00);
    }

    #[test]
    fn pinned_and_king_checked() {
      let board = Board::from_fen("k1n4R/8/1N6/8/8/1n6/8/K1N4r w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let knight = 0x20_00_00_00_00_00_00_00;
      let filter = movegen.pin_filter(knight) & movegen.check_filter();

      assert_eq!(movegen.knight(knight) & filter, 0);

      movegen.switch_turn();

      let knight = 0x00_00_00_00_00_00_00_20;
      let filter = movegen.pin_filter(knight) & movegen.check_filter();

      assert_eq!(movegen.knight(knight) & filter, 0);
    }
  }

  mod bishop {
    use crate::board::{Board, move_gen::MoveGen};

    #[test]
    fn default() {
      let board = Board::from_fen("k7/8/8/3b4/3B4/8/8/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let bishop = movegen.bishops & movegen.ally;
      assert_eq!(movegen.bishop(bishop), 0x02_44_28_00_28_44_82_01);

      movegen.switch_turn();
      let bishop = movegen.bishops & movegen.ally;
      assert_eq!(movegen.bishop(bishop), 0x01_82_44_28_00_28_44_02);
    }

    #[test]
    fn default_on_edge() {
      let board = Board::from_fen("kb6/8/8/8/8/8/8/KB6 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let bishop = movegen.bishops & movegen.ally;
      assert_eq!(movegen.bishop(bishop), 0x00_A0_10_08_04_02_01_00);
      movegen.switch_turn();
      let bishop = movegen.bishops & movegen.ally;
      assert_eq!(movegen.bishop(bishop), 0x00_01_02_04_08_10_A0_00);
    }

    #[test]
    fn pinned_vert() {
      let board = Board::from_fen("k6K/b6B/8/8/8/8/8/R6r w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let bishop = movegen.bishops & movegen.ally;
      let filter = movegen.pin_filter(bishop);

      assert_eq!(filter, 0x01_01_01_01_01_01_00_00);
      assert_eq!(movegen.bishop(bishop) & filter, 0);

      movegen.switch_turn();

      let bishop = movegen.bishops & movegen.ally;
      let filter = movegen.pin_filter(bishop);

      assert_eq!(filter, 0x80_80_80_80_80_80_00_00);
      assert_eq!(movegen.bishop(bishop) & filter, 0);
    }

    #[test]
    fn pinned_hor() {
      let board = Board::from_fen("kb5R/8/8/8/8/8/8/KB5r w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let bishop = movegen.bishops & movegen.ally;
      let filter = movegen.pin_filter(bishop);

      assert_eq!(filter, 0x3F_00_00_00_00_00_00_00);
      assert_eq!(movegen.bishop(bishop) & filter, 0);

      movegen.switch_turn();

      let bishop = movegen.bishops & movegen.ally;
      let filter = movegen.pin_filter(bishop);

      assert_eq!(filter, 0x00_00_00_00_00_00_00_3F);
      assert_eq!(movegen.bishop(bishop) & filter, 0);
    }

    #[test]
    fn pinned_diag() {
      let board = Board::from_fen("k6q/1b6/8/8/8/8/1B6/K6Q w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let bishop = movegen.bishops & movegen.ally;
      let filter = movegen.pin_filter(bishop);

      assert_eq!(filter, 0x00_00_20_10_08_04_02_01);
      assert_eq!(movegen.bishop(bishop) & filter, 0x00_00_20_10_08_04_02_01);

      movegen.switch_turn();

      let bishop = movegen.bishops & movegen.ally;
      let filter = movegen.pin_filter(bishop);

      assert_eq!(filter, 0x01_02_04_08_10_20_00_00);
      assert_eq!(movegen.bishop(bishop) & filter, 0x01_02_04_08_10_20_00_00);
    }

    #[test]
    fn king_checked() {
      let board = Board::from_fen("8/3b4/k6R/8/8/K6r/3B4/8 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let bishop = movegen.bishops & movegen.ally;
      let filter = movegen.check_filter();

      assert_eq!(movegen.bishop(bishop) & filter, 0x00_00_28_00_00_00_00_00);

      movegen.switch_turn();

      let bishop = movegen.bishops & movegen.ally;
      let filter = movegen.check_filter();

      assert_eq!(movegen.bishop(bishop) & filter, 0x00_00_00_00_00_28_00_00);
    }

    #[test]
    fn pinned_and_king_checked() {
      let board = Board::from_fen("k1b4R/1B6/8/8/8/8/1b6/K1B4r w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let bishop = 0x20_00_00_00_00_00_00_00;
      let filter = movegen.pin_filter(bishop) & movegen.check_filter();

      assert_eq!(movegen.bishop(bishop) & filter, 0);

      movegen.switch_turn();

      let bishop = 0x00_00_00_00_00_00_00_20;
      let filter = movegen.pin_filter(bishop) & movegen.check_filter();

      assert_eq!(movegen.bishop(bishop) & filter, 0);
    }
  }

  mod rook {
    use crate::board::{Board, move_gen::MoveGen};

    #[test]
    fn default() {
      let board = Board::from_fen("k7/8/8/3r4/4R3/8/8/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let rook = movegen.rooks & movegen.ally;
      assert_eq!(movegen.rook(rook), 0x08_08_08_F7_08_08_08_08);

      movegen.switch_turn();
      let rook = movegen.rooks & movegen.ally;
      assert_eq!(movegen.rook(rook), 0x10_10_10_10_EF_10_10_10);
    }

    #[test]
    fn default_on_edge() {
      let board = Board::from_fen("k6r/8/8/8/8/8/8/K6R w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let rook = movegen.rooks & movegen.ally;
      assert_eq!(movegen.rook(rook), 0x7E_01_01_01_01_01_01_01);
      movegen.switch_turn();
      let rook = movegen.rooks & movegen.ally;
      assert_eq!(movegen.rook(rook), 0x01_01_01_01_01_01_01_7E);
    }

    #[test]
    fn pinned_vert() {
      let board = Board::from_fen("k6K/r6R/8/8/8/8/8/Q6q w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let rook = movegen.rooks & movegen.ally;
      let filter = movegen.pin_filter(rook);

      assert_eq!(filter, 0x01_01_01_01_01_01_00_00);
      assert_eq!(movegen.rook(rook) & filter, 0x01_01_01_01_01_01_00_00);

      movegen.switch_turn();

      let rook = movegen.rooks & movegen.ally;
      let filter = movegen.pin_filter(rook);

      assert_eq!(filter, 0x80_80_80_80_80_80_00_00);
      assert_eq!(movegen.rook(rook) & filter, 0x80_80_80_80_80_80_00_00);
    }

    #[test]
    fn pinned_hor() {
      let board = Board::from_fen("kr5Q/8/8/8/8/8/8/KR5q w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let rook = movegen.rooks & movegen.ally;
      let filter = movegen.pin_filter(rook);

      assert_eq!(filter, 0x3F_00_00_00_00_00_00_00);
      assert_eq!(movegen.rook(rook) & filter, 0x3F_00_00_00_00_00_00_00);

      movegen.switch_turn();

      let rook = movegen.rooks & movegen.ally;
      let filter = movegen.pin_filter(rook);

      assert_eq!(filter, 0x00_00_00_00_00_00_00_3F);
      assert_eq!(movegen.rook(rook) & filter, 0x00_00_00_00_00_00_00_3F);
    }

    #[test]
    fn pinned_diag() {
      let board = Board::from_fen("k6b/1r6/8/8/8/8/1R6/K6B w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let rook = movegen.rooks & movegen.ally;
      let filter = movegen.pin_filter(rook);

      assert_eq!(filter, 0x00_00_20_10_08_04_02_01);
      assert_eq!(movegen.rook(rook) & filter, 0);

      movegen.switch_turn();

      let rook = movegen.rooks & movegen.ally;
      let filter = movegen.pin_filter(rook);

      assert_eq!(filter, 0x01_02_04_08_10_20_00_00);
      assert_eq!(movegen.rook(rook) & filter, 0);
    }

    #[test]
    fn king_checked() {
      let board = Board::from_fen("kr5b/8/8/8/8/8/8/KR5B w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let rook = movegen.rooks & movegen.ally;
      let filter = movegen.check_filter();

      assert_eq!(movegen.rook(rook) & filter, 0x00_40_00_00_00_00_00_00);

      movegen.switch_turn();

      let rook = movegen.rooks & movegen.ally;
      let filter = movegen.check_filter();

      assert_eq!(movegen.rook(rook) & filter, 0x00_00_00_00_00_00_40_00);
    }

    #[test]
    fn pinned_and_king_checked() {
      let board = Board::from_fen("kr4Qb/8/8/8/8/8/8/KR4qB w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let rook = movegen.rooks & movegen.ally;
      let filter = movegen.pin_filter(rook) & movegen.check_filter();

      assert_eq!(movegen.rook(rook) & filter, 0);

      movegen.switch_turn();

      let rook = movegen.rooks & movegen.ally;
      let filter = movegen.pin_filter(rook) & movegen.check_filter();

      assert_eq!(movegen.rook(rook) & filter, 0);
    }
  }

  mod queen {
    use crate::board::{Board, move_gen::MoveGen};

    #[test]
    fn default() {
      let board = Board::from_fen("k7/8/8/3q4/3Q4/8/8/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let queen = movegen.queens & movegen.ally;
      assert_eq!(movegen.queen(queen), 0x12_54_38_EF_38_44_82_01);

      movegen.switch_turn();
      let queen = movegen.queens & movegen.ally;
      assert_eq!(movegen.queen(queen), 0x01_82_44_38_EF_38_54_12);
    }

    #[test]
    fn default_on_edge() {
      let board = Board::from_fen("kq6/8/8/8/8/8/8/KQ6 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let queen = movegen.queens & movegen.ally;
      assert_eq!(movegen.queen(queen), 0x3F_E0_50_48_44_42_41_40);
      movegen.switch_turn();
      let queen = movegen.queens & movegen.ally;
      assert_eq!(movegen.queen(queen), 0x40_41_42_44_48_50_E0_3F);
    }

    #[test]
    fn pinned_vert() {
      let board = Board::from_fen("k6K/q6Q/8/8/8/8/8/R6r w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let queen = movegen.queens & movegen.ally;
      let filter = movegen.pin_filter(queen);

      assert_eq!(filter, 0x01_01_01_01_01_01_00_00);
      assert_eq!(movegen.queen(queen) & filter, 0x01_01_01_01_01_01_00_00);

      movegen.switch_turn();

      let queen = movegen.queens & movegen.ally;
      let filter = movegen.pin_filter(queen);

      assert_eq!(filter, 0x80_80_80_80_80_80_00_00);
      assert_eq!(movegen.queen(queen) & filter, 0x80_80_80_80_80_80_00_00);
    }

    #[test]
    fn pinned_hor() {
      let board = Board::from_fen("kq5R/8/8/8/8/8/8/KQ5r w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let queen = movegen.queens & movegen.ally;
      let filter = movegen.pin_filter(queen);

      assert_eq!(filter, 0x3F_00_00_00_00_00_00_00);
      assert_eq!(movegen.queen(queen) & filter, 0x3F_00_00_00_00_00_00_00);

      movegen.switch_turn();

      let queen = movegen.queens & movegen.ally;
      let filter = movegen.pin_filter(queen);

      assert_eq!(filter, 0x00_00_00_00_00_00_00_3F);
      assert_eq!(movegen.queen(queen) & filter, 0x00_00_00_00_00_00_00_3F);
    }

    #[test]
    fn pinned_diag() {
      let board = Board::from_fen("k6b/1q6/8/8/8/8/1Q6/K6B w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let queen = movegen.queens & movegen.ally;
      let filter = movegen.pin_filter(queen);

      assert_eq!(filter, 0x00_00_20_10_08_04_02_01);
      assert_eq!(movegen.queen(queen) & filter, 0x00_00_20_10_08_04_02_01);

      movegen.switch_turn();

      let queen = movegen.queens & movegen.ally;
      let filter = movegen.pin_filter(queen);

      assert_eq!(filter, 0x01_02_04_08_10_20_00_00);
      assert_eq!(movegen.queen(queen) & filter, 0x01_02_04_08_10_20_00_00);
    }

    #[test]
    fn king_checked() {
      let board = Board::from_fen("kq5b/8/8/8/8/8/8/KQ5B w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let queen = movegen.queens & movegen.ally;
      let filter = movegen.check_filter();

      assert_eq!(movegen.queen(queen) & filter, 0x00_40_00_00_00_00_00_00);

      movegen.switch_turn();

      let queen = movegen.queens & movegen.ally;
      let filter = movegen.check_filter();

      assert_eq!(movegen.queen(queen) & filter, 0x00_00_00_00_00_00_40_00);
    }

    #[test]
    fn pinned_and_king_checked() {
      let board = Board::from_fen("kq4Rb/8/8/8/8/8/8/KQ4rB w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let queen = movegen.queens & movegen.ally;
      let filter = movegen.pin_filter(queen) & movegen.check_filter();

      assert_eq!(movegen.queen(queen) & filter, 0);

      movegen.switch_turn();

      let queen = movegen.queens & movegen.ally;
      let filter = movegen.pin_filter(queen) & movegen.check_filter();

      assert_eq!(movegen.queen(queen) & filter, 0);
    }
  }

  mod king {
    use crate::board::{Board, move_gen::MoveGen};

    #[test]
    fn default() {
      let board = Board::from_fen("8/1k6/8/8/8/8/1K6/8 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0xE0_A0_E0_00_00_00_00_00
      );

      movegen.switch_turn();

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x00_00_00_00_00_E0_A0_E0
      );
    }

    #[test]
    fn default_on_edge() {
      let board = Board::from_fen("k7/8/8/8/8/8/8/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x40_C0_00_00_00_00_00_00
      );

      movegen.switch_turn();

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x00_00_00_00_00_00_C0_40
      );
    }

    #[test]
    fn castling() {
      let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
      let mut movegen = MoveGen::default(&board);

      assert_eq!(movegen.king_long_castle(), 0x20_00_00_00_00_00_00_00);
      assert_eq!(movegen.king_short_castle(), 0x02_00_00_00_00_00_00_00);

      movegen.switch_turn();

      assert_eq!(movegen.king_long_castle(), 0x00_00_00_00_00_00_00_20);
      assert_eq!(movegen.king_short_castle(), 0x00_00_00_00_00_00_00_02);
    }

    #[test]
    fn castling_no_rights() {
      let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      assert_eq!(movegen.king_long_castle(), 0);
      assert_eq!(movegen.king_short_castle(), 0);

      movegen.switch_turn();

      assert_eq!(movegen.king_long_castle(), 0);
      assert_eq!(movegen.king_short_castle(), 0);
    }

    #[test]
    fn castling_from_check() {
      let board = Board::from_fen("r3k2r/8/2B5/8/8/2b5/8/R3K2R w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      assert_eq!(movegen.king_long_castle(), 0);
      assert_eq!(movegen.king_short_castle(), 0);

      movegen.switch_turn();

      assert_eq!(movegen.king_long_castle(), 0);
      assert_eq!(movegen.king_short_castle(), 0);
    }

    #[test]
    fn castling_to_check() {
      let board = Board::from_fen("r3k2r/8/4B3/8/8/4b3/8/R3K2R w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      assert_eq!(movegen.king_long_castle(), 0);
      assert_eq!(movegen.king_short_castle(), 0);

      movegen.switch_turn();

      assert_eq!(movegen.king_long_castle(), 0);
      assert_eq!(movegen.king_short_castle(), 0);
    }

    #[test]
    fn castling_through_check() {
      let board = Board::from_fen("r3k2r/4B3/8/8/8/8/4b3/R3K2R w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      assert_eq!(movegen.king_long_castle(), 0);
      assert_eq!(movegen.king_short_castle(), 0);

      movegen.switch_turn();

      assert_eq!(movegen.king_long_castle(), 0);
      assert_eq!(movegen.king_short_castle(), 0);
    }

    #[test]
    fn checked_vert() {
      let board = Board::from_fen("r6R/8/8/8/8/8/8/K6k w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x40_40_00_00_00_00_00_00
      );

      movegen.switch_turn();

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x02_02_00_00_00_00_00_00
      );
    }

    #[test]
    fn checked_hor() {
      let board = Board::from_fen("k6R/8/8/8/8/8/8/K6r w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x00_C0_00_00_00_00_00_00
      );

      movegen.switch_turn();

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x00_00_00_00_00_00_C0_00
      );
    }

    #[test]
    fn checked_diag() {
      let board = Board::from_fen("k6b/8/8/8/8/8/8/K6B w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x40_80_00_00_00_00_00_00
      );

      movegen.switch_turn();

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x00_00_00_00_00_00_80_40
      );
    }

    #[test]
    fn around_pawn_danger() {
      let board = Board::from_fen("1k6/8/1P6/8/8/1p6/8/1K6 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0xA0_40_00_00_00_00_00_00
      );

      movegen.switch_turn();

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x00_00_00_00_00_00_40_A0
      );
    }

    #[test]
    fn around_knight_danger() {
      let board = Board::from_fen("k7/8/2N5/8/8/2n5/8/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x00_40_00_00_00_00_00_00
      );

      movegen.switch_turn();

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x00_00_00_00_00_00_40_00
      );
    }

    #[test]
    fn around_bishop_danger() {
      let board = Board::from_fen("1k5b/8/8/8/8/8/8/1K5B w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x20_A0_00_00_00_00_00_00
      );

      movegen.switch_turn();

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x00_00_00_00_00_00_A0_20
      );
    }

    #[test]
    fn around_rook_danger() {
      let board = Board::from_fen("k7/7R/1P6/8/8/8/7r/K7 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x40_00_00_00_00_00_00_00
      );

      movegen.switch_turn();

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x00_00_00_00_00_00_00_40
      );
    }
    #[test]
    fn around_queen_danger() {
      let board = Board::from_fen("1k6/3Q4/8/8/8/8/3q4/1K6 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x80_00_00_00_00_00_00_00
      );

      movegen.switch_turn();

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x00_00_00_00_00_00_00_80
      );
    }

    #[test]
    fn around_king_danger() {
      let board = Board::from_fen("8/8/8/1K1k4/8/8/8/8 w - - 0 1");
      let mut movegen = MoveGen::default(&board);

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x00_00_00_C0_80_C0_00_00
      );

      movegen.switch_turn();

      let king = movegen.kings & movegen.ally;
      assert_eq!(
        movegen.king_default(king) & !movegen.king_danger(),
        0x00_00_00_18_08_18_00_00
      );
    }
  }
}
