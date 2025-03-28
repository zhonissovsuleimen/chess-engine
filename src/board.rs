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

  pub fn move_piece(&mut self, from: u64, to: u64) -> bool {
    if (self.white_to_move && self.contains_black_piece(from))
      || (!self.white_to_move && self.contains_white_piece(from))
    {
      return false;
    }

    if !self.is_empty(to) {
      return false;
    }

    if self.white_to_move {
      for piece_sets in self.white.as_mut_array() {
        if (*piece_sets >> from & 1) == 1 {
          *piece_sets -= 1 << from;
          *piece_sets += 1 << to;
          self.white_to_move = false;
          return true;
        }
      }
    } else {
      for piece_sets in self.black.as_mut_array() {
        if (*piece_sets >> from & 1) == 1 {
          *piece_sets -= 1 << from;
          *piece_sets += 1 << to;
          self.white_to_move = true;
          return true;
        }
      }
    }

    return false;
  }

  pub fn is_empty(&self, id: u64) -> bool {
    return !(self.contains_white_piece(id) || self.contains_black_piece(id));
  }

  pub fn contains_white_piece(&self, id: u64) -> bool {
    return (self.white.concat() >> id & 1) == 1;
  }

  pub fn contains_black_piece(&self, id: u64) -> bool {
    return (self.black.concat() >> id & 1) == 1;
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
