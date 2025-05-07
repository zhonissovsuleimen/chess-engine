//todo: forced 3 fold repetition, simplified table, hashing
use bevy::ecs::system::Resource;

use super::{
  board_movement_trait::BoardMovement,
  cached_piece_moves::CachedPieceMoves,
  move_gen::MoveGen,
  move_input::{MoveInput, BISHOP, KNIGHT, QUEEN, ROOK},
  pieces::Pieces,
  status::*,
  util_fns::*,
};

#[derive(Resource)]
pub struct Board {
  pub white_turn: bool,
  pub white: Pieces,
  pub black: Pieces,
  clock: u64,
  half_clock: u64,
  pub cached_moves: CachedPieceMoves,

  pub(super) en_passant_mask: u64,
  pub(super) white_short_castle: bool,
  pub(super) white_long_castle: bool,
  pub(super) black_short_castle: bool,
  pub(super) black_long_castle: bool,
}

//constructor
impl Board {
  pub fn default() -> Board {
    Board {
      white: Pieces::white(),
      black: Pieces::black(),

      white_short_castle: true,
      white_long_castle: true,
      black_short_castle: true,
      black_long_castle: true,
      ..Board::empty()
    }
  }

  pub fn empty() -> Board {
    Board {
      white_turn: true,
      white: Pieces::empty(),
      black: Pieces::empty(),
      clock: 1,
      half_clock: 0,
      cached_moves: CachedPieceMoves::default(),

      en_passant_mask: 0,
      white_short_castle: false,
      white_long_castle: false,
      black_short_castle: false,
      black_long_castle: false,
    }
  }

  pub fn from_fen(fen_string: &str) -> Board {
    let mut board = Board::empty();

    let slices: Vec<&str> = fen_string.split_whitespace().collect();
    assert_eq!(slices.len(), 6);

    //pieces
    let pieces = slices[0];
    let rows: Vec<&str> = pieces.split('/').collect();
    assert_eq!(rows.len(), 8);

    for (row_id, row) in rows.iter().enumerate() {
      let mut total = 0;
      for c in row.chars() {
        let pos = (0x80_u64)
          .move_right_mask(total)
          .move_down_mask(row_id as u32);
        match c {
          'P' => board.white.pawns |= pos,
          'N' => board.white.knights |= pos,
          'B' => board.white.bishops |= pos,
          'R' => board.white.rooks |= pos,
          'Q' => board.white.queens |= pos,
          'K' => board.white.king = pos,
          'p' => board.black.pawns |= pos,
          'n' => board.black.knights |= pos,
          'b' => board.black.bishops |= pos,
          'r' => board.black.rooks |= pos,
          'q' => board.black.queens |= pos,
          'k' => board.black.king = pos,
          digit if c.is_ascii_digit() => {
            assert_ne!(digit, '0');
            assert_ne!(digit, '9');

            total += digit.to_digit(10).unwrap() - 1;
          }
          wrong_char => {
            panic!("Unexpected character ({wrong_char}) in piece placement data");
          }
        }
        total += 1;
      }
      assert_eq!(
        total,
        8,
        "The row {} contains {} squares",
        row_id + 1,
        total
      );
    }

    //turn
    match slices[1] {
      "w" => board.white_turn = true,
      "b" => board.white_turn = false,
      wrong_char => {
        panic!("Unexpected character ({wrong_char}) in active color data")
      }
    }

    //castling
    if slices[2] != "-" {
      let chars = slices[2].chars().collect::<Vec<char>>();

      for c in chars {
        match c {
          'K' => board.white_short_castle = true,
          'Q' => board.white_long_castle = true,
          'k' => board.black_short_castle = true,
          'q' => board.black_long_castle = true,
          wrong_char => {
            panic!("Unexpected character ({wrong_char}) in castling rights data")
          }
        }
      }
    }

    //en passant
    if slices[3] != "-" {
      let enp_chars = slices[3].chars().collect::<Vec<char>>();
      assert_eq!(enp_chars.len(), 2);
      let x = match enp_chars[0] {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        wrong_char => {
          panic!("Unexpected rank character ({wrong_char}) in en passant data")
        }
      };

      let y = match enp_chars[1] {
        '1' => 0,
        '2' => 1,
        '3' => 2,
        '4' => 3,
        '5' => 4,
        '6' => 5,
        '7' => 6,
        '8' => 7,
        wrong_char => {
          panic!("Unexpected file character ({wrong_char}) in en passant data")
        }
      };

      let shift = x + y * 8;
      board.en_passant_mask = 1 << shift;
    }

    board
  }
}

//moving/updating
impl Board {
  pub fn move_piece(&mut self, input: MoveInput) -> bool {
    let from_mask: u64 = input.from;
    let to_mask: u64 = input.to;

    self.update_cache(from_mask);
    let move_mask = to_mask & self.cached_moves.all();

    //order matters
    self.handle_en_passant(move_mask);
    self.handle_pawn_advance(move_mask);
    self.handle_castling(from_mask, move_mask);
    self.handle_move(from_mask, move_mask, input.promotion);

    self.update_clocks(move_mask);
    self.white_turn ^= move_mask > 0;
    move_mask > 0
  }

  fn update_clocks(&mut self, move_mask: u64) {
    let pawn_moved = move_mask & self.cached_moves.pawn_default > 0 || self.en_passant_mask > 0;
    let capture_or_pawn = mask_from_bool(pawn_moved || move_mask & self.cached_moves.capturing > 0);
    self.half_clock = if_bool(
      move_mask > 0,
      !capture_or_pawn & (self.half_clock + 1),
      self.half_clock,
    );

    self.clock += (move_mask > 0 && !self.white_turn) as u64;
  }

  pub fn update_cache(&mut self, from_mask: u64) {
    if from_mask != self.cached_moves.from_mask {
      self.cached_moves = MoveGen::cached(&self, from_mask);
    }
  }

  fn handle_en_passant(&mut self, move_mask: u64) {
    let en_passanted = mask_from_bool(move_mask & self.en_passant_mask > 0);
    let en_passanted_pawn = en_passanted
      & if_bool(
        self.white_turn,
        move_mask.move_down_mask(1),
        move_mask.move_up_mask(1),
      );

    self.white.remove_piece(en_passanted_pawn);
    self.black.remove_piece(en_passanted_pawn);
  }

  fn handle_pawn_advance(&mut self, move_mask: u64) {
    let pawn_advanced = move_mask & self.cached_moves.pawn_advance > 0;

    self.en_passant_mask = if_bool(
      self.white_turn,
      move_mask.move_down_mask(1),
      move_mask.move_up_mask(1),
    ) & mask_from_bool(pawn_advanced);
  }

  fn handle_castling(&mut self, from_mask: u64, move_mask: u64) {
    let white_king_moved = self.white_turn && move_mask & self.cached_moves.king_default > 0;
    let white_long_rook_moved = self.white_turn
      && move_mask & self.cached_moves.rook > 0
      && from_mask == 0x80_00_00_00_00_00_00_00;
    let white_short_rook_moved = self.white_turn
      && move_mask & self.cached_moves.rook > 0
      && from_mask == 0x01_00_00_00_00_00_00_00;

    self.white_long_castle &= !(white_king_moved || white_long_rook_moved);
    self.white_short_castle &= !(white_king_moved || white_short_rook_moved);

    let black_king_moved = !self.white_turn && move_mask & self.cached_moves.king_default > 0;
    let black_long_rook_moved = !self.white_turn
      && move_mask & self.cached_moves.rook > 0
      && from_mask == 0x00_00_00_00_00_00_00_80;
    let black_short_rook_moved = !self.white_turn
      && move_mask & self.cached_moves.rook > 0
      && from_mask == 0x00_00_00_00_00_00_00_01;

    self.black_long_castle &= !(black_king_moved || black_long_rook_moved);
    self.black_short_castle &= !(black_king_moved || black_short_rook_moved);
  }

  fn handle_move(&mut self, from_mask: u64, move_mask: u64, promotion_choice: u64) {
    self.white.remove_piece(move_mask);
    self.black.remove_piece(move_mask);

    self.white.move_piece(from_mask, move_mask);
    self.black.move_piece(from_mask, move_mask);

    let pawn_moves =
      self.cached_moves.pawn_default | self.cached_moves.pawn_advance | self.cached_moves.capturing;
    let promoting_mask = 0xFF_00_00_00_00_00_00_FF;
    let promoted = pawn_moves & promoting_mask > 0;

    let knight_promotion = mask_from_bool(promoted && promotion_choice == KNIGHT);
    let bishop_promotion = mask_from_bool(promoted && promotion_choice == BISHOP);
    let rook_promotion = mask_from_bool(promoted && promotion_choice == ROOK);
    let queen_promotion = mask_from_bool(promoted && promotion_choice == QUEEN);

    self.white.promote_to_knight(knight_promotion & move_mask);
    self.white.promote_to_bishop(bishop_promotion & move_mask);
    self.white.promote_to_rook(rook_promotion & move_mask);
    self.white.promote_to_queen(queen_promotion & move_mask);
    self.black.promote_to_knight(knight_promotion & move_mask);
    self.black.promote_to_bishop(bishop_promotion & move_mask);
    self.black.promote_to_rook(rook_promotion & move_mask);
    self.black.promote_to_queen(queen_promotion & move_mask);
  }
}

//state
impl Board {
  pub(crate) fn pawns(&self) -> u64 {
    self.white.pawns | self.black.pawns
  }

  pub(crate) fn knights(&self) -> u64 {
    self.white.knights | self.black.knights
  }

  pub(crate) fn bishops(&self) -> u64 {
    self.white.bishops | self.black.bishops
  }

  pub(crate) fn rooks(&self) -> u64 {
    self.white.rooks | self.black.rooks
  }

  pub(crate) fn queens(&self) -> u64 {
    self.white.queens | self.black.queens
  }

  pub(crate) fn kings(&self) -> u64 {
    self.white.king | self.black.king
  }

  pub fn get_status(&self) -> u64 {
    let mut movegen = MoveGen::default(&self);
    let piece_status = movegen.get_status();
    let fifty_move = mask_from_bool(self.half_clock > 100);

    let result = (fifty_move & DRAW) | (!fifty_move & piece_status);
    result
  }

  pub fn is_empty(&self, at_mask: u64) -> bool {
    self.white.is_empty(at_mask) && self.black.is_empty(at_mask)
  }

  pub fn is_promotion(&self, to_mask: u64) -> bool {
    let pawn_moves =
      self.cached_moves.pawn_default | self.cached_moves.pawn_advance | self.cached_moves.capturing;
    let promoting_mask = 0xFF_00_00_00_00_00_00_FF;
    pawn_moves & promoting_mask & to_mask > 0
  }
}

#[cfg(test)]
mod tests {
  mod constructors {
    use crate::board::{Board, status::PLAYING};

    #[test]
    fn default() {
      let board = Board::default();

      assert_eq!(board.white.pawns, 0x00_FF_00_00_00_00_00_00);
      assert_eq!(board.white.knights, 0x42_00_00_00_00_00_00_00);
      assert_eq!(board.white.bishops, 0x24_00_00_00_00_00_00_00);
      assert_eq!(board.white.rooks, 0x81_00_00_00_00_00_00_00);
      assert_eq!(board.white.queens, 0x10_00_00_00_00_00_00_00);
      assert_eq!(board.white.king, 0x08_00_00_00_00_00_00_00);
      assert_eq!(board.black.pawns, 0x00_00_00_00_00_00_FF_00);
      assert_eq!(board.black.knights, 0x00_00_00_00_00_00_00_42);
      assert_eq!(board.black.bishops, 0x00_00_00_00_00_00_00_24);
      assert_eq!(board.black.rooks, 0x00_00_00_00_00_00_00_81);
      assert_eq!(board.black.queens, 0x00_00_00_00_00_00_00_10);
      assert_eq!(board.black.king, 0x00_00_00_00_00_00_00_08);

      assert_eq!(board.white_turn, true);
      assert_eq!(board.get_status(), PLAYING);
      assert_eq!(board.clock, 1);
      assert_eq!(board.half_clock, 0);

      assert_eq!(board.en_passant_mask, 0x00_00_00_00_00_00_00_00);
      assert_eq!(board.white_long_castle, true);
      assert_eq!(board.white_short_castle, true);
      assert_eq!(board.black_long_castle, true);
      assert_eq!(board.black_short_castle, true);
    }

    #[test]
    fn fen_default() {
      let a = Board::default();
      let b = Board::from_fen(r"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

      assert_eq!(a.white.pawns, b.white.pawns);
      assert_eq!(a.white.knights, b.white.knights);
      assert_eq!(a.white.bishops, b.white.bishops);
      assert_eq!(a.white.rooks, b.white.rooks);
      assert_eq!(a.white.queens, b.white.queens);
      assert_eq!(a.white.king, b.white.king);
      assert_eq!(a.black.pawns, b.black.pawns);
      assert_eq!(a.black.knights, b.black.knights);
      assert_eq!(a.black.bishops, b.black.bishops);
      assert_eq!(a.black.rooks, b.black.rooks);
      assert_eq!(a.black.queens, b.black.queens);
      assert_eq!(a.black.king, b.black.king);

      assert_eq!(a.white_turn, b.white_turn);
      assert_eq!(a.get_status(), b.get_status());
      assert_eq!(a.clock, b.clock);
      assert_eq!(a.half_clock, b.half_clock);

      assert_eq!(a.en_passant_mask, b.en_passant_mask);
      assert_eq!(a.white_long_castle, b.white_long_castle);
      assert_eq!(a.white_short_castle, b.white_short_castle);
      assert_eq!(a.black_long_castle, b.black_long_castle);
      assert_eq!(a.black_short_castle, b.black_short_castle);
    }

    #[test]
    #[should_panic]
    fn fen_incorrect_row_data() {
      Board::from_fen(r"rnbqkbnr/pppppppp/8/8/7/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    }

    #[test]
    #[should_panic]
    fn fen_only_piece_data() {
      Board::from_fen(r"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
    }
  }

  mod clock {
    use crate::board::{Board, move_input::MoveInput};

    #[test]
    fn full_clock() {
      let mut board = Board::default();
      assert_eq!(board.clock, 1);

      assert_eq!(board.move_piece(MoveInput::from_id(52, 36)), true);
      assert_eq!(board.clock, 1);
      assert_eq!(board.move_piece(MoveInput::from_id(12, 4)), false);
      assert_eq!(board.clock, 1);
      assert_eq!(board.move_piece(MoveInput::from_id(12, 28)), true);
      assert_eq!(board.clock, 2);

      assert_eq!(board.move_piece(MoveInput::from_id(61, 34)), true);
      assert_eq!(board.clock, 2);
      assert_eq!(board.move_piece(MoveInput::from_id(56, 57)), false);
      assert_eq!(board.clock, 2);
      assert_eq!(board.move_piece(MoveInput::from_id(1, 18)), true);
      assert_eq!(board.clock, 3);
    }

    #[test]
    fn half_clock() {
      let mut board = Board::default();
      assert_eq!(board.half_clock, 0);

      assert_eq!(board.move_piece(MoveInput::from_id(52, 36)), true);
      assert_eq!(board.half_clock, 0);
      assert_eq!(board.move_piece(MoveInput::from_id(12, 4)), false);
      assert_eq!(board.half_clock, 0);
      assert_eq!(board.move_piece(MoveInput::from_id(12, 28)), true);
      assert_eq!(board.half_clock, 0);

      assert_eq!(board.move_piece(MoveInput::from_id(61, 34)), true);
      assert_eq!(board.half_clock, 1);
      assert_eq!(board.move_piece(MoveInput::from_id(56, 57)), false);
      assert_eq!(board.half_clock, 1);
      assert_eq!(board.move_piece(MoveInput::from_id(1, 18)), true);
      assert_eq!(board.half_clock, 2);

      assert_eq!(board.move_piece(MoveInput::from_id(34, 20)), true);
      assert_eq!(board.half_clock, 3);
      assert_eq!(board.move_piece(MoveInput::from_id(55, 47)), false);
      assert_eq!(board.half_clock, 3);
      assert_eq!(board.move_piece(MoveInput::from_id(4, 20)), true);
      assert_eq!(board.half_clock, 0);
    }
  }
}
