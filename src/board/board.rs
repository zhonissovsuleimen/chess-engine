//todo: forced 3 fold repetition, simplified table, hashing, promoting lOl
use bevy::ecs::system::Resource;

use super::{board_movement_trait::BoardMovement, cached_piece_moves::CachedPieceMoves, move_gen::MoveGen, pieces::Pieces, status::*, util_fns::*};

#[derive(Resource)]
pub struct Board {
  pub status: u64,
  pub white_turn: bool,
  pub white: Pieces,
  pub black: Pieces,
  clock: u64,
  half_clock: u64,
  moves: CachedPieceMoves,

  pub(super) advance_mask: u64,
  pub(super) en_passant_mask: u64,
  pub(super) castling_mask: u64,
}

//constructor
impl Board {
  pub fn default() -> Board {
    let mut board = Board {
      white: Pieces::white(),
      black: Pieces::black(),

      ..Board::empty()
    };

    board.initialize_masks();
    board
  }

  pub fn empty() -> Board {
    Board {
      status: PLAYING,
      white_turn: true,
      white: Pieces::empty(),
      black: Pieces::empty(),
      clock: 1,
      half_clock: 0,

      advance_mask: 0,
      en_passant_mask: 0,
      castling_mask: 0,
      moves: CachedPieceMoves::default(),
    }
  }

  pub fn from_fen(fen_string: &str) -> Board {
    let mut board = Board::empty();

    let slices: Vec<&str> = fen_string.split_whitespace().collect();
    assert_eq!(slices.len(), 6);

    //pieces
    let pieces = slices[0];

    let mut rank = 7;
    let mut file = 0;
    for c in pieces.chars() {
      let mut increment_file = true;

      // let id = (7 - rank) * 8 + file;
      let pos = 0x80_u64.move_right_mask(file).move_down_mask(7 - rank);
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

          file += digit.to_digit(10).unwrap();
          increment_file = false;
        }
        '/' => {
          rank -= 1;
          file = 0;

          increment_file = false;
        }
        wrong_char => {
          panic!("Unexpected character ({wrong_char}) in piece placement data");
        }
      }

      if increment_file {
        file += 1;
      }
    }

    //advance mask
    board.advance_mask = board.white.pawns & 0x00_FF_00_00_00_00_00_00;
    board.advance_mask |= board.black.pawns & 0x00_00_00_00_00_00_FF_00;

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
          'K' => board.castling_mask |= 0x09_00_00_00_00_00_00_00,
          'Q' => board.castling_mask |= 0x88_00_00_00_00_00_00_00,
          'k' => board.castling_mask |= 0x00_00_00_00_00_00_00_09,
          'q' => board.castling_mask |= 0x00_00_00_00_00_00_00_88,
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
  pub fn move_piece(&mut self, from_id: usize, to_id: usize) -> bool {
    let from_mask: u64 = if_bool(from_id < 64, 1 << from_id, 0);
    let to_mask: u64 = if_bool(to_id < 64, 1 << to_id, 0);
    self.moves = MoveGen::cached(&self, from_mask);
    self.update_status();
    let playing = mask_from_bool(self.status == PLAYING);
    
    let move_mask = to_mask & self.moves.all() & playing;

    //order matters
    self.handle_en_passant(move_mask);
    self.handle_pawn_advance(from_mask, move_mask);
    self.handle_castling(from_mask, move_mask);
    self.handle_move(from_mask, move_mask);

    self.update_clocks(move_mask);
    self.white_turn ^= move_mask > 0;
    move_mask > 0
  }

  fn initialize_masks(&mut self) {
    self.advance_mask = self.white.pawns | self.black.pawns;
    self.en_passant_mask = 0;
    self.castling_mask = self.white.king | self.white.rooks | self.black.king | self.black.rooks;
  }

  fn update_status(&mut self) {
    let piece_status = self.moves.status;
    let fifty_move = mask_from_bool(self.half_clock > 100);

    self.status = (fifty_move & DRAW) | (!fifty_move & piece_status);
  }

  fn update_clocks(&mut self, move_mask: u64) {
    let pawn_moved = move_mask & self.moves.pawn_default > 0 || self.en_passant_mask > 0;
    let capture_or_pawn = mask_from_bool(pawn_moved || move_mask & self.moves.capturing > 0);
    self.half_clock = if_bool(
      move_mask > 0,
      !capture_or_pawn & (self.half_clock + 1),
      self.half_clock,
    );

    self.clock += (move_mask > 0 && !self.white_turn) as u64;
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

  fn handle_pawn_advance(&mut self, from_mask: u64, move_mask: u64) {
    let pawn_advance_move = self.moves.pawn_advance;
    let pawn_advanced = mask_from_bool(move_mask & pawn_advance_move > 0);
    self.advance_mask &= !(pawn_advanced & from_mask);
    self.en_passant_mask = if_bool(
      move_mask > 0,
      if_bool(
        self.white_turn,
        pawn_advance_move.move_down_mask(1),
        pawn_advance_move.move_up_mask(1),
      ),
      self.en_passant_mask,
    );
  }

  fn handle_castling(&mut self, from_mask: u64, move_mask: u64) {
    let king_move = self.moves.king_default;
    let rook_move = self.moves.rook;
    let revoke_castling_move = move_mask & (king_move | rook_move);
    self.castling_mask &= !revoke_castling_move;

    let long_castled = move_mask & self.moves.king_long_castle;
    let short_castled = move_mask & self.moves.king_short_castle;

    let rook_from = long_castled.move_left_mask(2) | short_castled.move_right_mask(1);
    let rook_to = long_castled.move_right_mask(1) | short_castled.move_left_mask(1);

    self.white.move_piece(rook_from, rook_to);
    self.black.move_piece(rook_from, rook_to);

    let castled = mask_from_bool(long_castled | short_castled > 0);
    self.castling_mask &= !(castled & from_mask);
  }

  fn handle_move(&mut self, from_mask: u64, move_mask: u64) {
    self.white.remove_piece(move_mask);
    self.black.remove_piece(move_mask);
    self.white.move_piece(from_mask, move_mask);
    self.black.move_piece(from_mask, move_mask);
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

  pub(crate) fn get_piece_moves(&self, at_mask: u64) -> u64 {
    let moves = MoveGen::cached(&self, at_mask);
    moves.all()
  }
}

#[cfg(test)]
mod tests {
  mod constructors {
    use crate::{board::status::PLAYING, *};

    #[test]
    fn default() {
      let board = Board::default();

      assert_eq!(board.white.pieces_concat(), 0xFF_FF_00_00_00_00_00_00);
      assert_eq!(board.black.pieces_concat(), 0xFF_FF);
      assert_eq!(board.white_turn, true);
      assert_eq!(board.status, PLAYING);
      assert_eq!(board.clock, 1);
      assert_eq!(board.half_clock, 0);

      assert_eq!(board.advance_mask, 0x00_FF_00_00_00_00_FF_00);
      assert_eq!(board.en_passant_mask, 0x00_00_00_00_00_00_00_00);
      assert_eq!(board.castling_mask, 0x89_00_00_00_00_00_00_89);
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
      assert_eq!(a.status, b.status);
      assert_eq!(a.clock, b.clock);
      assert_eq!(a.half_clock, b.half_clock);

      assert_eq!(a.advance_mask, b.advance_mask);
      assert_eq!(a.en_passant_mask, b.en_passant_mask);
      assert_eq!(a.castling_mask, b.castling_mask);
    }
  }
}
