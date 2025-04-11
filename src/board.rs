//TODO: castiling, checks, checkmates?
use crate::{
  board_movement_trait::BoardMovement,
  pieces::Pieces,
  util_fns::{branchless_if, mask_from_bool},
};
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

  empty_mask: u64,
  advance_mask: u64,
  en_passant_mask: u64,
  enemy_capturing_moves: u64,
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
      wrong_char => {
        panic!("Unexpected character ({wrong_char}) in active color data")
      }
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

    //calculating moves
    let pawn_advance_move = self.gen_pawn_advance_move(from_mask & self.pawns());
    let move_mask = to_mask & self.get_piece_moves(from_mask);

    //handling en passant logic
    let en_passanted = move_mask & self.en_passant_mask > 0;
    let to_remove = branchless_if(
      en_passanted,
      branchless_if(
        self.white_turn,
        move_mask.move_down_mask(1),
        move_mask.move_up_mask(1),
      ),
      0,
    );
    self.white.remove_piece(to_remove);
    self.black.remove_piece(to_remove);

    //handling pawn advance logic
    let pawn_advanced = move_mask & pawn_advance_move > 0;
    self.advance_mask &= !(branchless_if(pawn_advanced, from_mask, 0));
    self.en_passant_mask = branchless_if(
      pawn_advanced,
      branchless_if(
        self.white_turn,
        move_mask.move_down_mask(1),
        move_mask.move_up_mask(1),
      ),
      0,
    );

    //moving
    self.black.remove_piece(move_mask);
    self.white.remove_piece(move_mask);

    self.white.move_piece(from_mask, move_mask);
    self.black.move_piece(from_mask, move_mask);

    self.white_turn ^= move_mask > 0;
    self.update_masks();

    return move_mask > 0;
  }

  fn update_masks(&mut self) {
    self.empty_mask = !(self.white.pieces_concat() | self.black.pieces_concat());
    self.advance_mask = self.white.pawns_advance | self.black.pawns_advance;

    let turn = self.white_turn;
    self.white_turn = true;
    let white_capturing_moves = self.gen_pawn_capturing_moves(self.white.pawns, true)
      | self.gen_knight_moves(self.white.knights)
      | self.gen_bishop_moves(self.white.bishops)
      | self.gen_rook_moves(self.white.rooks)
      | self.gen_queen_moves(self.white.queens)
      | self.gen_king_moves(self.white.king);

    self.white_turn = false;
    let black_capturing_moves = self.gen_pawn_capturing_moves(self.black.pawns, true)
      | self.gen_knight_moves(self.black.knights)
      | self.gen_bishop_moves(self.black.bishops)
      | self.gen_rook_moves(self.black.rooks)
      | self.gen_queen_moves(self.black.queens)
      | self.gen_king_moves(self.black.king);

    self.enemy_capturing_moves = branchless_if(
      self.white_turn,
      black_capturing_moves,
      white_capturing_moves,
    );
    self.white_turn = turn;
  }
}

//move generations
impl Board {
  fn gen_pawn_default_move(&self, at_mask: u64) -> u64 {
    let pawn_default_move = branchless_if(
      self.white_turn,
      at_mask.move_up_mask(1),
      at_mask.move_down_mask(1),
    );

    pawn_default_move & self.empty_mask
  }

  fn gen_pawn_advance_move(&self, at_mask: u64) -> u64 {
    let default = self.gen_pawn_default_move(at_mask);
    let can_default_mask = branchless_if(
      self.white_turn,
      default.move_up_mask(1),
      default.move_down_mask(1),
    );

    let can_advance_mask = branchless_if(
      self.white_turn,
      self.advance_mask.move_up_mask(2),
      self.advance_mask.move_down_mask(2),
    );

    let pawn_advance_move = branchless_if(
      self.white_turn,
      at_mask.move_up_mask(2),
      at_mask.move_down_mask(2),
    );

    pawn_advance_move & self.empty_mask & can_advance_mask & can_default_mask
  }

  fn gen_pawn_capturing_moves(&self, at_mask: u64, ignore_enemy_check: bool) -> u64 {
    let enemy_mask = branchless_if(
      self.white_turn,
      self.black.pieces_concat(),
      self.white.pieces_concat(),
    );
    let ignore_mask = mask_from_bool(ignore_enemy_check);

    let one_move_up = at_mask.move_up_mask(1);
    let one_move_down = at_mask.move_down_mask(1);

    let enemy_to_left = branchless_if(
      self.white_turn,
      one_move_up.move_left_mask(1),
      one_move_down.move_left_mask(1),
    );
    let enemy_to_right = branchless_if(
      self.white_turn,
      one_move_up.move_right_mask(1),
      one_move_down.move_right_mask(1),
    );

    (enemy_to_left | enemy_to_right) & (ignore_mask | enemy_mask | self.en_passant_mask)
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
    let enemy_mask = branchless_if(
      self.white_turn,
      self.black.pieces_concat(),
      self.white.pieces_concat(),
    );

    let mut moves = 0;

    for (dx, dy) in offsets {
      let mut new_move = branchless_if(
        dx > 0,
        at_mask.move_right_mask(dx.abs() as u32),
        at_mask.move_left_mask(dx.abs() as u32),
      );

      new_move = branchless_if(
        dy > 0,
        new_move.move_up_mask(dy.abs() as u32),
        new_move.move_down_mask(dy.abs() as u32),
      );

      moves |= new_move & (self.empty_mask | enemy_mask);
    }

    moves
  }

  fn gen_iterative_moves(&self, at_mask: u64, dx: i32, dy: i32) -> u64 {
    let enemy_mask = branchless_if(
      self.white_turn,
      self.black.pieces_concat(),
      self.white.pieces_concat(),
    );

    let mut moves = 0;
    let mut current = (dx, dy);
    loop {
      let mut new_move = branchless_if(
        dx > 0,
        at_mask.move_right_mask(current.0.abs() as u32),
        at_mask.move_left_mask(current.0.abs() as u32),
      );
      new_move = branchless_if(
        dy > 0,
        new_move.move_up_mask(current.1.abs() as u32),
        new_move.move_down_mask(current.1.abs() as u32),
      );

      current = (current.0 + dx, current.1 + dy);

      let within_board = new_move > 0;
      let is_enemy = new_move & enemy_mask > 0;
      let is_empty = new_move & self.empty_mask > 0;
      let is_ally = !(is_empty || is_enemy);

      moves |= branchless_if(is_enemy || is_empty, new_move, 0);

      if !within_board || is_ally || is_enemy {
        break;
      }
    }

    moves
  }

  fn gen_pin_path(&self, at_mask: u64, dx: i32, dy: i32) -> u64 {
    let enemy_mask = branchless_if(
      self.white_turn,
      self.black.pieces_concat(),
      self.white.pieces_concat(),
    );
    let white_attackers = self.white.bishops | self.white.rooks | self.white.queens;
    let black_attackers = self.black.bishops | self.black.rooks | self.black.queens;
    let correct_pieces = branchless_if(self.white_turn, black_attackers, white_attackers);

    let mut pin_path = 0;
    let mut current = (dx, dy);

    let mut ally_count = 0;
    let mut enemy_count = 0;
    let mut pinned;

    loop {
      let mut new_move = branchless_if(
        dx > 0,
        at_mask.move_right_mask(current.0.abs() as u32),
        at_mask.move_left_mask(current.0.abs() as u32),
      );
      new_move = branchless_if(
        dy > 0,
        new_move.move_up_mask(current.1.abs() as u32),
        new_move.move_down_mask(current.1.abs() as u32),
      );

      current = (current.0 + dx, current.1 + dy);

      let within_board = new_move > 0;
      let is_enemy = new_move & enemy_mask > 0;
      let is_empty = new_move & self.empty_mask > 0;
      let is_ally = !(is_empty || is_enemy);

      enemy_count += is_enemy as u64;
      ally_count += is_ally as u64;

      pinned = ally_count == 1 && enemy_count == 1 && (correct_pieces & new_move > 0);
      let done = (ally_count == 0 && enemy_count == 1)
        || (ally_count == 2 && enemy_count == 0)
        || pinned;

      pin_path |= new_move;

      if !within_board || done {
        break;
      }
    }

    branchless_if(pinned, pin_path, 0)
  }

  fn gen_pin_mask(&self, at_mask: u64) -> u64 {
    let ally_king = branchless_if(self.white_turn, self.white.king, self.black.king);
    let mut pin_path = 0;

    pin_path |= self.gen_pin_path(ally_king, -1, -1);
    pin_path |= self.gen_pin_path(ally_king, 1, -1);
    pin_path |= self.gen_pin_path(ally_king, -1, 1);
    pin_path |= self.gen_pin_path(ally_king, 1, 1);
    pin_path |= self.gen_pin_path(ally_king, -1, 0);
    pin_path |= self.gen_pin_path(ally_king, 0, -1);
    pin_path |= self.gen_pin_path(ally_king, 1, 0);
    pin_path |= self.gen_pin_path(ally_king, 0, 1);

    branchless_if(pin_path & at_mask > 0, pin_path, u64::MAX)
  }
}

//state
impl Board {
  fn pawns(&self) -> u64 {
    self.white.pawns | self.black.pawns
  }

  fn knights(&self) -> u64 {
    self.white.knights | self.black.knights
  }

  fn bishops(&self) -> u64 {
    self.white.bishops | self.black.bishops
  }

  fn rooks(&self) -> u64 {
    self.white.rooks | self.black.rooks
  }

  fn queens(&self) -> u64 {
    self.white.queens | self.black.queens
  }

  fn kings(&self) -> u64 {
    self.white.king | self.black.king
  }

  pub fn get_piece_delta(&self) -> i32 {
    self.white.get_value() - self.black.get_value()
  }

  pub fn get_piece_moves(&self, at_mask: u64) -> u64 {
    let pawn_default_move = self.gen_pawn_default_move(at_mask & self.pawns());
    let pawn_advance_move = self.gen_pawn_advance_move(at_mask & self.pawns());
    let pawn_capturing_move =
      self.gen_pawn_capturing_moves(at_mask & self.pawns(), false);
    let pawn_moves = pawn_default_move | pawn_advance_move | pawn_capturing_move;
    let knight_moves = self.gen_knight_moves(at_mask & self.knights());
    let bishop_moves = self.gen_bishop_moves(at_mask & self.bishops());
    let rook_moves = self.gen_rook_moves(at_mask & self.rooks());
    let queen_moves = self.gen_queen_moves(at_mask & self.queens());
    let king_moves =
      self.gen_king_moves(at_mask & self.kings()) & !self.enemy_capturing_moves;

    let pseudo =
      pawn_moves | knight_moves | bishop_moves | rook_moves | queen_moves | king_moves;

    let pin_filter = self.gen_pin_mask(at_mask);

    pin_filter & pseudo
  }
}

impl Default for Board {
  fn default() -> Board {
    let mut board = Board {
      white_turn: true,
      white: Pieces::white(),
      black: Pieces::black(),

      status: Default::default(),
      empty_mask: Default::default(),
      advance_mask: Default::default(),
      en_passant_mask: Default::default(),
      enemy_capturing_moves: Default::default(),
    };

    board.update_masks();
    board
  }
}
