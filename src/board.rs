use crate::pieces::Pieces;
use bevy::ecs::system::Resource;

#[derive(Resource)]
pub struct Board {
  pub white_to_move: bool,
  pub white: Pieces,
  pub black: Pieces,

  white_turn_mask: u64,
  empty_mask: u64,
  enemy_mask: u64,
  advance_mask: u64,
}

impl Board {
  pub fn empty() -> Board {
    let mut board = Board {
      white_to_move: true,
      white: Pieces::empty(),
      black: Pieces::empty(),

      white_turn_mask: 0,
      empty_mask: 0,
      enemy_mask: 0,
      advance_mask: 0,
    };

    board.update_masks();
    board
  }

  pub fn default() -> Board {
    let mut board = Board {
      white_to_move: true,
      white: Pieces::white(),
      black: Pieces::black(),

      white_turn_mask: 0,
      empty_mask: 0,
      enemy_mask: 0,
      advance_mask: 0,
    };

    board.update_masks();
    board
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
      let move_one = self.gen_pawn_moves(from_mask);
      let move_two = self.gen_pawn_advence_move(from_mask);

      new_moves_mask += move_one & to_mask;
      if move_two & to_mask > 0 {
        new_moves_mask += move_two;
        if self.white_to_move {
          self.white.remove_advance(from_mask);
        } else {
          self.black.remove_advance(from_mask);
        }
      }
    } else if self.is_knight(from_mask) {
      let moves = self.gen_knight_moves(from_mask);
      if moves & to_mask > 0 {
        new_moves_mask += moves;
      }
    } else if self.is_bishop(from_mask) {
      let moves = self.gen_bishop_moves(from_mask);
      if moves & to_mask > 0 {
        new_moves_mask += moves;
      }
    } else if self.is_rook(from_mask) {
      let moves = self.gen_rook_moves(from_mask);
      if moves & to_mask > 0 {
        new_moves_mask += moves;
      }
    } else if self.is_queen(from_mask) {
      let moves = self.gen_queen_moves(from_mask);
      if moves & to_mask > 0 {
        new_moves_mask += moves;
      }
    } else if self.is_king(from_mask) {
      let moves = self.gen_king_moves(from_mask);
      if moves & to_mask > 0 {
        new_moves_mask += moves;
      }
    }

    if (to_mask & new_moves_mask) > 0 {
      if self.white_to_move {
        self.black.remove_piece(to_mask);
        self.white.move_piece(from_mask, to_mask);
      } else {
        self.white.remove_piece(to_mask);
        self.black.move_piece(from_mask, to_mask);
      }

      self.white_to_move = !self.white_to_move;
      self.update_masks();
      return true;
    }

    return false;
  }

  fn update_masks(&mut self) {
    self.white_turn_mask = !((self.white_to_move as u64).overflowing_sub(1).0);
    self.empty_mask = !(self.white.pieces_concat() | self.black.pieces_concat());
    self.enemy_mask = self.white_turn_mask & self.black.pieces_concat()
      | !self.white_turn_mask & self.white.pieces_concat();
    self.advance_mask = self.white.pawns_advance | self.black.pawns_advance;
  }

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

  fn gen_pawn_moves(&self, at_mask: u64) -> u64 {
    let mut pawn_move;

    if self.white_to_move {
      pawn_move = at_mask.checked_shr(8).unwrap_or(0) & self.empty_mask;
    } else {
      pawn_move = at_mask.checked_shl(8).unwrap_or(0) & self.empty_mask;
    }

    pawn_move |= self.gen_pawn_capturing_moves(at_mask);
    return pawn_move;
  }

  fn gen_pawn_advence_move(&self, at_mask: u64) -> u64 {
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

    let can_advance = (at_mask & self.advance_mask) > 0;
    pawn_move = (both_moves & self.empty_mask & move_two) * can_advance as u64;

    pawn_move
  }

  fn gen_pawn_capturing_moves(&self, at_mask: u64) -> u64 {
    let pawn_move;

    let x = at_mask.trailing_zeros();

    //todo: remove brach?
    if self.white_to_move {
      let enemy_to_left_mask = at_mask.checked_shr(9).unwrap_or(0);
      let left_edge_check = (x != 0) as u64;
      let can_take_left_mask = (enemy_to_left_mask & !self.empty_mask) * left_edge_check;

      let enemy_to_right_mask = at_mask.checked_shr(7).unwrap_or(0);
      let right_edge_check = (x != 7) as u64;
      let can_take_right_mask = (enemy_to_right_mask & !self.empty_mask) * right_edge_check;

      pawn_move = can_take_left_mask + can_take_right_mask;
    } else {
      let enemy_to_left_mask = at_mask.checked_shl(7).unwrap_or(0);
      let left_edge_check = (x != 0) as u64;
      let can_take_left_mask = (enemy_to_left_mask & !self.empty_mask) * left_edge_check;

      let enemy_to_right_mask = at_mask.checked_shl(9).unwrap_or(0);
      let right_edge_check = (x != 7) as u64;
      let can_take_right_mask = (enemy_to_right_mask & !self.empty_mask) * right_edge_check;

      pawn_move = can_take_left_mask + can_take_right_mask;
    }

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

    let id = at_mask.trailing_zeros() as i32;
    let x = id % 8;
    let y = id / 8;

    let mut current = (dx, dy);
    loop {
      let new_x = x + current.0;
      let new_y = y + current.1;
      let new_pos = new_x + new_y * 8;

      let within_board = new_x >= 0 && new_x < 8 && new_y >= 0 && new_y < 8;

      let shift = (within_board as i32) * new_pos + (!within_board as i32) * 64; // new_pos if within board, 64 otherwise (will lead to 0 mask);
      let new_move_mask = 1u64.checked_shl(shift as u32).unwrap_or(0);

      let capture = new_move_mask & self.enemy_mask > 0;
      let friendly_piece = new_move_mask & !(self.enemy_mask | self.empty_mask) > 0;

      if !within_board || friendly_piece {
        return moves;
      } else if capture {
        moves |= new_move_mask & self.enemy_mask;
        return moves;
      } else {
        moves |= new_move_mask & self.empty_mask;
      }

      current = (current.0 + dx, current.1 + dy);
    }
  }

  pub fn get_piece_delta(&self) -> i32 {
    self.white.get_value() - self.black.get_value()
  }

  pub fn from_fen(fen_string: &str) -> Board {
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
      "w" => board.white_to_move = true,
      "b" => board.white_to_move = false,
      wrong_char => panic!("Unexpected character ({wrong_char}) in active color data"),
    }

    board.update_masks();
    board
  }
}
