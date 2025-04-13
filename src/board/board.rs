use crate::board::{move_generation::Modifier, util_fns::mask_from_bool};

//TODO: changing game status, from_fen update
use super::{
  board_movement_trait::BoardMovement, pieces::Pieces, util_fns::branchless_if,
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

  //updated during update_masks
  pub(super) empty_mask: u64,
  pub(super) under_attack_mask: u64,

  //should be initialized and updated during move_piece
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
      status: Status::Playing,
      white_turn: true,
      white: Pieces::empty(),
      black: Pieces::empty(),

      empty_mask: 0,
      advance_mask: 0,
      en_passant_mask: 0,
      under_attack_mask: 0,
      castling_mask: 0,
    }
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
    let pawn_advance_move =
      self.gen_pawn_advance_move(from_mask & self.pawns(), Modifier::NONE);
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

    //handling castling logic
    let long_castled = self.gen_king_long_castle_moves(from_mask, Modifier::NONE) > 0;
    let short_castled = self.gen_king_short_castle_moves(from_mask, Modifier::NONE) > 0;
    let castled = long_castled || short_castled;
    
    let ally_king = branchless_if(self.white_turn, self.white.king, self.black.king);

    let rook_castle_from = branchless_if(castled, branchless_if(long_castled, ally_king.move_left_mask(4), ally_king.move_right_mask(3)), 0);
    let rook_castle_to = branchless_if(castled, branchless_if(long_castled, ally_king.move_left_mask(1), ally_king.move_right_mask(1)), 0);
    
    let rook_or_king_move = self.gen_default_king_moves(from_mask, Modifier::NONE) | self.gen_rook_moves(from_mask, Modifier::NONE);
    self.castling_mask &= !branchless_if(castled, ally_king, 0);
    self.castling_mask &= !branchless_if(to_mask & rook_or_king_move > 0, from_mask, 0);

    //moving
    self.black.remove_piece(move_mask);
    self.white.remove_piece(move_mask);

    self.white.move_piece(from_mask, move_mask);
    self.black.move_piece(from_mask, move_mask);

    let white_rook_from = mask_from_bool(self.white_turn) & rook_castle_from;
    let black_rook_from = !mask_from_bool(self.white_turn) & rook_castle_from;
    self.white.move_piece(white_rook_from, rook_castle_to);
    self.black.move_piece(black_rook_from, rook_castle_to);

    self.white_turn ^= move_mask > 0;
    self.update_masks();
    return move_mask > 0;
  }

  fn initialize_masks(&mut self) {
    self.advance_mask = self.white.pawns | self.black.pawns;
    self.en_passant_mask = 0;
    self.castling_mask =
      self.white.king | self.white.rooks | self.black.king | self.black.rooks;

    self.update_masks();
  }

  fn update_masks(&mut self) {
    self.empty_mask = !(self.white.pieces_concat() | self.black.pieces_concat());


    let pieces = branchless_if(self.white_turn, self.black.pieces_concat(), self.white.pieces_concat());

    self.under_attack_mask = self
      .gen_pawn_capturing_moves(self.pawns() & pieces, Modifier::NO_ENEMY_CHECK)
      | self.gen_knight_moves(self.knights() & pieces, Modifier::NONE)
      | self.gen_bishop_moves(self.bishops() & pieces, Modifier::NONE)
      | self.gen_rook_moves(self.rooks() & pieces, Modifier::NONE)
      | self.gen_queen_moves(self.queens() & pieces, Modifier::NONE)
      | self.gen_default_king_moves(self.kings() & pieces, Modifier::NONE);

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

  pub fn get_piece_moves(&self, at_mask: u64) -> u64 {
    let pawn_default_move =
      self.gen_pawn_default_move(at_mask & self.pawns(), Modifier::NONE);
    let pawn_advance_move =
      self.gen_pawn_advance_move(at_mask & self.pawns(), Modifier::NONE);
    let pawn_capturing_move =
      self.gen_pawn_capturing_moves(at_mask & self.pawns(), Modifier::NONE);
    let pawn_moves = pawn_default_move | pawn_advance_move | pawn_capturing_move;
    let knight_moves = self.gen_knight_moves(at_mask & self.knights(), Modifier::NONE);
    let bishop_moves = self.gen_bishop_moves(at_mask & self.bishops(), Modifier::NONE);
    let rook_moves = self.gen_rook_moves(at_mask & self.rooks(), Modifier::NONE);
    let queen_moves = self.gen_queen_moves(at_mask & self.queens(), Modifier::NONE);
    let king_default_move = self.gen_default_king_moves(at_mask & self.kings(), Modifier::NONE) & !self.under_attack_mask;
    let king_long_castle = self.gen_king_long_castle_moves(at_mask & self.kings(), Modifier::NONE);
    let king_short_castle = self.gen_king_short_castle_moves(at_mask & self.kings(), Modifier::NONE);
    let king_moves = king_default_move | king_long_castle | king_short_castle;

    let pseudo =
      pawn_moves | knight_moves | bishop_moves | rook_moves | queen_moves | king_moves;

    let pin_filter = self.gen_pin_filter(at_mask);
    let check_filter = self.gen_check_filter(at_mask);

    pseudo & pin_filter & check_filter
  }
}
