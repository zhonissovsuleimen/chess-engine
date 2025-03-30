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

    let white_to_move = self.white_to_move;
    let emptiness = self.empty_mask();
    let selected_pieces = if white_to_move {
      &mut self.white
    } else {
      &mut self.black
    };

    let mut new_moves_mask: u64 = 0;

    if selected_pieces.is_pawn(from_mask) {
      let mut pawn_moves = 0;
      let shift_fn = if white_to_move {
        |mask: u64, shift: u32| mask.checked_shr(shift).unwrap_or(0)
      } else {
        |mask: u64, shift: u32| mask.checked_shl(shift).unwrap_or(0)
      };

      let move_one = shift_fn(from_mask, 8);
      let move_two = shift_fn(from_mask, 16);
      let both_moves = move_one | move_two;

      pawn_moves += move_one & emptiness;

      //todo: bitwise trickery
      if emptiness & both_moves > 0 && selected_pieces.can_advance(from_mask) {
        pawn_moves += move_two;
        selected_pieces.remove_advance(from_mask);
      }

      new_moves_mask = pawn_moves;
    } else if selected_pieces.is_knight(from_mask) {
      let mut knight_moves = 0;

      knight_moves += from_mask.checked_shr(17).unwrap_or(0);
      knight_moves += from_mask.checked_shr(15).unwrap_or(0);
      knight_moves += from_mask.checked_shr(10).unwrap_or(0);
      knight_moves += from_mask.checked_shr(6).unwrap_or(0);
      knight_moves += from_mask.checked_shl(6).unwrap_or(0);
      knight_moves += from_mask.checked_shl(10).unwrap_or(0);
      knight_moves += from_mask.checked_shl(15).unwrap_or(0);
      knight_moves += from_mask.checked_shl(17).unwrap_or(0);

      new_moves_mask = knight_moves & emptiness;
    } else if selected_pieces.is_bishop(from_mask) {
      let mut bishop_moves = 0;

      let mut tl = from_mask.checked_shr(9).unwrap_or(0);
      while emptiness & tl > 0 {
        bishop_moves += tl;
        tl = tl.checked_shr(9).unwrap_or(0);
      }

      let mut tr = from_mask.checked_shr(7).unwrap_or(0);
      while emptiness & tr > 0 {
        bishop_moves += tr;
        tr = tr.checked_shr(7).unwrap_or(0);
      }

      let mut bl = from_mask.checked_shl(7).unwrap_or(0);
      while emptiness & bl > 0 {
        bishop_moves += bl;
        bl = bl.checked_shl(7).unwrap_or(0);
      }

      let mut br = from_mask.checked_shl(9).unwrap_or(0);
      while emptiness & br > 0 {
        bishop_moves += br;
        br = br.checked_shl(9).unwrap_or(0);
      }

      new_moves_mask += bishop_moves;
    } else if selected_pieces.is_rook(from_mask) {
    } else if selected_pieces.is_queen(from_mask) {
    } else if selected_pieces.is_king(from_mask) {
      let mut king_moves = 0;
      king_moves += from_mask.checked_shr(9).unwrap_or(0);
      king_moves += from_mask.checked_shr(8).unwrap_or(0);
      king_moves += from_mask.checked_shr(7).unwrap_or(0);
      king_moves += from_mask.checked_shr(1).unwrap_or(0);
      king_moves += from_mask.checked_shl(1).unwrap_or(0);
      king_moves += from_mask.checked_shl(7).unwrap_or(0);
      king_moves += from_mask.checked_shl(8).unwrap_or(0);
      king_moves += from_mask.checked_shl(9).unwrap_or(0);

      new_moves_mask = king_moves & emptiness;
    }

    if to_mask & new_moves_mask > 0 {
      selected_pieces.r#move(from_mask, to_mask);

      self.white_to_move = !self.white_to_move;
      return true;
    }

    return false;
  }

  fn is_empty(&self, at_mask: u64) -> bool {
    return self.white.is_empty(at_mask) && self.black.is_empty(at_mask);
  }

  fn empty_mask(&self) -> u64 {
    return !(self.white.pieces_concat() | self.white.pieces_concat());
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
