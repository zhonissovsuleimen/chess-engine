//TODO: castiling, pin, checks, checkmates?
use crate::{board_movement_trait::BoardMovement, pieces::Pieces};
use bevy::ecs::system::Resource;

#[derive(PartialEq, Default)]
pub enum Status {
  #[default]
  Playing,
  WhiteWon,
  BlackWon,
}

#[derive(Resource)]
pub struct Board {
  pub status: Status,
  pub white_turn: bool,
  pub white: Pieces,
  pub black: Pieces,

  white_turn_mask: u64,

  empty_mask: u64,
  enemy_mask: u64,
  advance_mask: u64,
  en_passant_mask: u64,
}

//constructor
impl Board {
  pub fn empty() -> Board {
    let mut board = Board {
      white: Pieces::empty(),
      black: Pieces::empty(),

      ..Default::default()
    };

    board.update_masks();
    board
  }

  pub fn from_fen(fen_string: &str) -> Board {
    //todo: pawn advance mask depending on rank
    let mut board = Board::empty();

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
      "w" => board.white_turn = true,
      "b" => board.white_turn = false,
      wrong_char => panic!("Unexpected character ({wrong_char}) in active color data"),
    }

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
        wrong_char => panic!("Unexpected rank character ({wrong_char}) in en passant data"),
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
        wrong_char => panic!("Unexpected file character ({wrong_char}) in en passant data"),
      };

      let shift = x + y * 8;
      board.en_passant_mask = 1 << shift;
    }

    board.update_masks();
    board
  }
}

//moving/updating
impl Board {
  pub fn move_piece(&mut self, from_id: usize, to_id: usize) -> bool {
    assert!(from_id < 64);
    assert!(to_id < 64);

    let from_mask: u64 = 1 << from_id;
    let to_mask: u64 = 1 << to_id;

    if self.status != Status::Playing {
      return false;
    }

    if (self.white_turn && self.white.is_empty(from_mask))
      || (!self.white_turn && self.black.is_empty(from_mask))
    {
      return false;
    }

    let mut new_moves_mask = 0;

    if self.is_pawn(from_mask) {
      let default_move = self.gen_pawn_default_move(from_mask);
      let advance_move = self.gen_pawn_advance_move(from_mask);
      let capturing_move = self.gen_pawn_capturing_moves(from_mask);

      new_moves_mask |= (capturing_move | default_move) & to_mask;

      if (advance_move & to_mask) > 0 {
        new_moves_mask |= advance_move & to_mask;
        self.en_passant_mask = default_move;
        if self.white_turn {
          self.white.remove_advance(from_mask);
        } else {
          self.black.remove_advance(from_mask);
        }
      }
    } else if self.is_knight(from_mask) {
      new_moves_mask |= self.gen_knight_moves(from_mask);
    } else if self.is_bishop(from_mask) {
      new_moves_mask |= self.gen_bishop_moves(from_mask);
    } else if self.is_rook(from_mask) {
      new_moves_mask |= self.gen_rook_moves(from_mask);
    } else if self.is_queen(from_mask) {
      new_moves_mask |= self.gen_queen_moves(from_mask);
    } else if self.is_king(from_mask) {
      new_moves_mask |= self.gen_king_moves(from_mask);
    }

    if (to_mask & new_moves_mask) > 0 {
      if self.white_turn {
        self.black.remove_piece(to_mask);
        self.white.move_piece(from_mask, to_mask);
      } else {
        self.white.remove_piece(to_mask);
        self.black.move_piece(from_mask, to_mask);
      }

      if to_mask & self.en_passant_mask > 0 {
        if self.white_turn {
          let pawn_mask = self.en_passant_mask.move_down_mask(1);
          self.black.remove_piece(pawn_mask);
        } else {
          let pawn_mask = self.en_passant_mask.move_up_mask(1);
          self.white.remove_piece(pawn_mask);
        }

        self.en_passant_mask = 0;
      }

      self.white_turn = !self.white_turn;
      self.update_masks();
      return true;
    }

    return false;
  }

  fn update_masks(&mut self) {
    self.white_turn_mask = !((self.white_turn as u64).overflowing_sub(1).0);
    self.empty_mask = !(self.white.pieces_concat() | self.black.pieces_concat());
    self.enemy_mask = self.white_turn_mask & self.black.pieces_concat()
      | !self.white_turn_mask & self.white.pieces_concat();
    self.advance_mask = self.white.pawns_advance | self.black.pawns_advance;
  }
}

//move generations
impl Board {
  fn gen_pawn_default_move(&self, at_mask: u64) -> u64 {
    let mut pawn_default_move = self.white_turn_mask & at_mask.move_up_mask(1)
      | !self.white_turn_mask & at_mask.move_down_mask(1);

    pawn_default_move &= self.empty_mask;

    pawn_default_move
  }

  fn gen_pawn_advance_move(&self, at_mask: u64) -> u64 {
    let mut pawn_advance_move = self.white_turn_mask & at_mask.move_up_mask(2)
      | !self.white_turn_mask & at_mask.move_down_mask(2);

    let color_adjusted_empty_mask = self.empty_mask
      & (self.white_turn_mask & self.empty_mask.move_up_mask(1)
        | !self.white_turn_mask & self.empty_mask.move_down_mask(1));

    let color_adjusted_advance_mask = self.white_turn_mask & self.advance_mask.move_up_mask(2)
      | !self.white_turn_mask & self.advance_mask.move_down_mask(2);

    pawn_advance_move &= color_adjusted_empty_mask & color_adjusted_advance_mask;

    pawn_advance_move
  }

  fn gen_pawn_capturing_moves(&self, at_mask: u64) -> u64 {
    let mut pawn_move = 0;

    let enemy_to_left = self.white_turn_mask & at_mask.move_up_mask(1).move_left_mask(1)
      | !self.white_turn_mask & at_mask.move_down_mask(1).move_left_mask(1);

    let enemy_to_right = self.white_turn_mask & at_mask.move_up_mask(1).move_right_mask(1)
      | !self.white_turn_mask & at_mask.move_down_mask(1).move_right_mask(1);

    pawn_move |= (enemy_to_left | enemy_to_right) & (self.enemy_mask | self.en_passant_mask);

    pawn_move
  }

  fn gen_knight_moves(&self, at_mask: u64) -> u64 {
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

    self.gen_offset_moves(at_mask, offsets)
  }

  fn gen_bishop_moves(&self, at_mask: u64) -> u64 {
    let mut moves = 0;

    moves |= self.gen_iterative_moves(at_mask, -1, -1);
    moves |= self.gen_iterative_moves(at_mask, 1, -1);
    moves |= self.gen_iterative_moves(at_mask, -1, 1);
    moves |= self.gen_iterative_moves(at_mask, 1, 1);

    moves
  }

  fn gen_rook_moves(&self, at_mask: u64) -> u64 {
    let mut moves = 0;

    moves |= self.gen_iterative_moves(at_mask, -1, 0);
    moves |= self.gen_iterative_moves(at_mask, 0, -1);
    moves |= self.gen_iterative_moves(at_mask, 1, 0);
    moves |= self.gen_iterative_moves(at_mask, 0, 1);

    moves
  }

  fn gen_queen_moves(&self, at_mask: u64) -> u64 {
    self.gen_bishop_moves(at_mask) | self.gen_rook_moves(at_mask)
  }

  fn gen_king_moves(&self, at_mask: u64) -> u64 {
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

    self.gen_offset_moves(at_mask, offsets)
  }

  fn gen_offset_moves(&self, at_mask: u64, offsets: Vec<(i32, i32)>) -> u64 {
    let mut moves = 0;

    let id = at_mask.trailing_zeros() as i32;
    let x = id % 8;
    let y = id / 8;

    for (dx, dy) in offsets {
      let new_x = x + dx;
      let new_y = y + dy;
      let new_pos = new_x + new_y * 8;

      let within_board = new_x >= 0 && new_x < 8 && new_y >= 0 && new_y < 8;
      let shift = (within_board as i32) * new_pos + (!within_board as i32) * 64; // new_pos if within board, 64 otherwise (will lead to 0 mask);

      let new_move_mask = 1u64.checked_shl(shift as u32).unwrap_or(0);

      moves |= new_move_mask & (self.empty_mask | self.enemy_mask);
    }

    moves
  }

  fn gen_iterative_moves(&self, at_mask: u64, dx: i32, dy: i32) -> u64 {
    let mut moves = 0;

    let x_positive = (dx > 0) as u64;
    let x_positive_mask = !(x_positive.overflowing_sub(1).0);

    let y_positive = (dy > 0) as u64;
    let y_positive_mask = !(y_positive.overflowing_sub(1).0);

    let mut current = (dx, dy);
    loop {
      let mut new_move_mask = x_positive_mask & at_mask.move_right_mask(current.0.abs() as u32)
        | !x_positive_mask & at_mask.move_left_mask(current.0.abs() as u32);

      new_move_mask = y_positive_mask & new_move_mask.move_up_mask(current.1.abs() as u32)
        | !y_positive_mask & new_move_mask.move_down_mask(current.1.abs() as u32);

      let friend_mask = new_move_mask & !(self.empty_mask | self.enemy_mask);

      let capture_mask = new_move_mask & self.enemy_mask;
      if new_move_mask == 0 || friend_mask > 0 {
        break;
      } else if capture_mask > 0 {
        moves |= capture_mask;
        break;
      }

      moves |= new_move_mask;

      current = (current.0 + dx, current.1 + dy);
    }

    moves
  }
}

//state
impl Board {
  fn is_pawn(&self, at_mask: u64) -> bool {
    return self.white.is_pawn(at_mask) || self.black.is_pawn(at_mask);
  }

  fn is_knight(&self, at_mask: u64) -> bool {
    return self.white.is_knight(at_mask) || self.black.is_knight(at_mask);
  }

  fn is_bishop(&self, at_mask: u64) -> bool {
    return self.white.is_bishop(at_mask) || self.black.is_bishop(at_mask);
  }

  fn is_rook(&self, at_mask: u64) -> bool {
    return self.white.is_rook(at_mask) || self.black.is_rook(at_mask);
  }

  fn is_queen(&self, at_mask: u64) -> bool {
    return self.white.is_queen(at_mask) || self.black.is_queen(at_mask);
  }

  fn is_king(&self, at_mask: u64) -> bool {
    return self.white.is_king(at_mask) || self.black.is_king(at_mask);
  }

  pub fn get_piece_delta(&self) -> i32 {
    self.white.get_value() - self.black.get_value()
  }
}

impl Default for Board {
  fn default() -> Board {
    let mut board = Board {
      white_turn: true,
      white: Pieces::white(),
      black: Pieces::black(),

      status: Default::default(),
      white_turn_mask: Default::default(),
      empty_mask: Default::default(),
      enemy_mask: Default::default(),
      advance_mask: Default::default(),
      en_passant_mask: Default::default(),
    };

    board.update_masks();
    board
  }
}
