use crate::pieces::Pieces;
use bevy::ecs::system::Resource;

#[derive(Resource)]
pub struct Board {
  pub white_to_move: bool,
  pub white: Pieces,
  pub black: Pieces,
}

impl Board {
  pub fn default() -> Board {
    Board {
      white_to_move: true,
      white: Pieces::white(),
      black: Pieces::black(),
    }
  }

  pub fn move_piece(&mut self, from_id: usize, to_id: usize) -> bool {
    assert!(from_id < 64);
    assert!(to_id < 64);

    let from_mask: u64 = 1 << from_id;
    let to_mask: u64 = 1 << to_id;

    if (self.white_to_move && self.white.is_empty(from_mask))
      || (!self.white_to_move && self.black.is_empty(from_mask))
    {
      return false;
    }

    let mut new_moves_mask = 0;

    if self.is_pawn(from_mask) {
      let move_one = self.gen_pawn_move(from_mask);
      let move_two = self.gen_pawn_advence_move(from_mask);
      let capturing_moves = self.gen_pawn_capturing_moves(from_mask);

      new_moves_mask += move_one & to_mask;
      if move_two & to_mask > 0 {
        new_moves_mask += move_two;
        if self.white_to_move {
          self.white.remove_advance(from_mask);
        } else {
          self.black.remove_advance(from_mask);
        }
      }
      if capturing_moves & to_mask > 0{
        new_moves_mask += capturing_moves;
        if self.white_to_move {
          self.black.remove_piece(to_mask);
        } else {
          self.white.remove_piece(to_mask);
        }
      }
    }

    if (to_mask & new_moves_mask) > 0 {
      if self.white_to_move {
        self.white.move_piece(from_mask, to_mask);
      } else {
        self.black.move_piece(from_mask, to_mask);
      }

      self.white_to_move = !self.white_to_move;
      return true;
    }

    return false;
  }

  fn is_empty(&self, at_mask: u64) -> bool {
    return self.white.is_empty(at_mask) && self.black.is_empty(at_mask);
  }

  fn empty_mask(&self) -> u64 {
    return !(self.white.pieces_concat() | self.black.pieces_concat());
  }

  fn advance_mask(&self) -> u64 {
    return self.white.pawns_advance | self.black.pawns_advance;
  }

  fn is_pawn(&self, at_mask: u64) -> bool {
    return self.white.is_pawn(at_mask) || self.black.is_pawn(at_mask);
  }

  pub fn is_knight(&self, at_mask: u64) -> bool {
    return self.white.is_knight(at_mask) || self.black.is_knight(at_mask);
  }

  pub fn is_bishop(&self, at_mask: u64) -> bool {
    return self.white.is_bishop(at_mask) || self.black.is_bishop(at_mask);
  }

  pub fn is_rook(&self, at_mask: u64) -> bool {
    return self.white.is_rook(at_mask) || self.black.is_rook(at_mask);
  }

  pub fn is_queen(&self, at_mask: u64) -> bool {
    return self.white.is_queen(at_mask) || self.black.is_queen(at_mask);
  }

  pub fn is_king(&self, at_mask: u64) -> bool {
    return self.white.is_king(at_mask) || self.black.is_king(at_mask);
  }

  pub fn gen_pawn_move(&self, at_mask: u64) -> u64 {
    let pawn_move;

    if self.white_to_move {
      pawn_move = at_mask.checked_shr(8).unwrap_or(0) & self.empty_mask();
    } else {
      pawn_move = at_mask.checked_shl(8).unwrap_or(0) & self.empty_mask();
    }

    return pawn_move;
  }

  pub fn gen_pawn_advence_move(&self, at_mask: u64) -> u64 {
    let pawn_move;
    let (move_one, move_two) = if self.white_to_move {
      (
        at_mask.checked_shr(8).unwrap_or(0),
        at_mask.checked_shr(16).unwrap_or(0),
      )
    } else {
      (
        at_mask.checked_shl(8).unwrap_or(0),
        at_mask.checked_shl(16).unwrap_or(0),
      )
    };

    let both_moves = move_one | move_two;

    let can_advance = (at_mask & self.advance_mask()) > 0;
    pawn_move = (both_moves & self.empty_mask() & move_two) * can_advance as u64;

    pawn_move
  }

  pub fn gen_pawn_capturing_moves(&self, at_mask: u64) -> u64 {
    let pawn_move;

    let x = at_mask.trailing_zeros();

    //todo: remove brach?
    if self.white_to_move {
      let enemy_to_left_mask = at_mask.checked_shr(9).unwrap_or(0);
      let left_edge_check = (x != 0) as u64;
      let can_take_left_mask = (enemy_to_left_mask & !self.empty_mask()) * left_edge_check;

      let enemy_to_right_mask = at_mask.checked_shr(7).unwrap_or(0);
      let right_edge_check = (x != 7) as u64;
      let can_take_right_mask = (enemy_to_right_mask & !self.empty_mask()) * right_edge_check;

      pawn_move = can_take_left_mask + can_take_right_mask;
    } else {
      let enemy_to_left_mask = at_mask.checked_shl(7).unwrap_or(0);
      let left_edge_check = (x != 0) as u64;
      let can_take_left_mask = (enemy_to_left_mask & !self.empty_mask()) * left_edge_check;

      let enemy_to_right_mask = at_mask.checked_shl(9).unwrap_or(0);
      let right_edge_check = (x != 7) as u64;
      let can_take_right_mask = (enemy_to_right_mask & !self.empty_mask()) * right_edge_check;

      pawn_move = can_take_left_mask + can_take_right_mask;
    }

    pawn_move
  }

  pub fn get_piece_delta(&self) -> i32 {
    self.white.get_value() - self.black.get_value()
  }

  pub fn from_fen(fen_string: &str) -> Board {
    let mut board = Board {
      white_to_move: true,
      white: Pieces::empty(),
      black: Pieces::empty(),
    };

    let slices: Vec<&str> = fen_string.split_whitespace().collect();
    assert_eq!(slices.len(), 6);

    let pieces = slices[0];

    let mut rank = 7;
    let mut file = 0;
    for c in pieces.chars() {
      let mut increment_file = true;

      let bit_shift = (7 - rank) * 8 + file;
      match c {
        'P' => board.white.pawns += 1 << bit_shift,
        'N' => board.white.knights += 1 << bit_shift,
        'B' => board.white.bishops += 1 << bit_shift,
        'R' => board.white.rooks += 1 << bit_shift,
        'Q' => board.white.queens += 1 << bit_shift,
        'K' => board.white.king = 1 << bit_shift,
        'p' => board.black.pawns += 1 << bit_shift,
        'n' => board.black.knights += 1 << bit_shift,
        'b' => board.black.bishops += 1 << bit_shift,
        'r' => board.black.rooks += 1 << bit_shift,
        'q' => board.black.queens += 1 << bit_shift,
        'k' => board.black.king = 1 << bit_shift,
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

    match slices[1] {
      "w" => board.white_to_move = true,
      "b" => board.white_to_move = false,
      wrong_char => panic!("Unexpected character ({wrong_char}) in active color data"),
    }

    board
  }
}
