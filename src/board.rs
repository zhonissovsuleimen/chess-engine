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

    let correct_pieces = if self.white_to_move {
      &mut self.white
    } else {
      &mut self.black
    };

    for i in 0..6 {
      let mut new_moves_mask: u64 = 0;
      match i {
        0 => {
          if self.white_to_move {
            new_moves_mask += from_mask.checked_shr(8).unwrap_or(0);
          } else {
            new_moves_mask += from_mask.checked_shl(8).unwrap_or(0);
          }
        }
        _ => {}
      }

      if to_mask & new_moves_mask > 0 {
        correct_pieces.r#move(from_mask, to_mask);

        self.white_to_move = !self.white_to_move;
        return true;
      }
    }

    return false;
  }

  fn is_empty(&self, at_mask: u64) -> bool {
    return self.white.is_empty(at_mask) && self.black.is_empty(at_mask);
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
