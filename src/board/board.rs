//toVdo forced 3 fold repetition, simplified table, hashing, promoting lOl
use super::{
  board_movement_trait::BoardMovement, move_generation_modifiers::*, pieces::Pieces, status::*,
  util_fns::*,
};
use bevy::ecs::system::Resource;

#[derive(Resource)]
pub struct Board {
  pub status: u64,
  pub white_turn: bool,
  pub white: Pieces,
  pub black: Pieces,
  clock: u64,
  half_clock: u64,

  //updated during update_masks
  pub(super) white_turn_mask: u64,
  pub(super) ally: u64,
  pub(super) enemy: u64,
  pub(super) empty: u64,
  pub(super) king_danger: u64,
  pub(super) ally_king: u64,

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
      status: PLAYING,
      white_turn: true,
      white: Pieces::empty(),
      black: Pieces::empty(),
      clock: 1,
      half_clock: 0,

      white_turn_mask: u64::MAX,
      ally: 0,
      enemy: 0,
      empty: 0,
      king_danger: 0,
      ally_king: 0,

      advance_mask: 0,
      en_passant_mask: 0,
      castling_mask: 0,
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

      let bit_shift = (7 - rank) * 8 + file;
      let pos = 1 << bit_shift;
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
          'K' => board.castling_mask |= 0x90_00_00_00_00_00_00_00,
          'Q' => board.castling_mask |= 0x11_00_00_00_00_00_00_00,
          'k' => board.castling_mask |= 0x00_00_00_00_00_00_00_90,
          'q' => board.castling_mask |= 0x00_00_00_00_00_00_00_11,
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

    board.update_masks();
    board
  }
}

//moving/updating
impl Board {
  pub fn move_piece(&mut self, from_id: usize, to_id: usize) -> bool {
    let playing = self.status == PLAYING;
    let from_mask: u64 = if_bool(playing && from_id < 64, 1 << from_id, 0) & self.ally;
    let to_mask: u64 = if_bool(playing && to_id < 64, 1 << to_id, 0) & !self.ally;

    let move_mask = to_mask & self.get_piece_moves(from_mask);

    //order matters
    self.handle_en_passant(move_mask);
    self.handle_pawn_advance(from_mask, move_mask);
    self.handle_castling(from_mask, move_mask);
    self.handle_move(from_mask, move_mask);

    self.update_clocks(from_mask, move_mask);
    self.white_turn ^= move_mask > 0;
    self.update_masks();
    self.update_status();
    move_mask > 0
  }

  fn initialize_masks(&mut self) {
    self.advance_mask = self.white.pawns | self.black.pawns;
    self.en_passant_mask = 0;
    self.castling_mask = self.white.king | self.white.rooks | self.black.king | self.black.rooks;

    self.update_masks();
  }

  fn update_masks(&mut self) {
    self.white_turn_mask = mask_from_bool(self.white_turn);
    self.ally = if_mask(
      self.white_turn_mask,
      self.white.pieces_concat(),
      self.black.pieces_concat(),
    );
    self.enemy = if_mask(
      self.white_turn_mask,
      self.black.pieces_concat(),
      self.white.pieces_concat(),
    );
    self.empty = !(self.ally | self.enemy);
    self.ally_king = if_mask(self.white_turn_mask, self.white.king, self.black.king);

    self.king_danger = self
      .gen_pawn_capturing_moves(self.pawns() & self.enemy, ALLY_IS_ENEMY | EMPTY_IS_ENEMY)
      | self.gen_knight_moves(self.knights() & self.enemy, ALLY_IS_ENEMY)
      | self.gen_bishop_moves(self.bishops() & self.enemy, ALLY_IS_ENEMY | KING_IS_EMPTY)
      | self.gen_rook_moves(self.rooks() & self.enemy, ALLY_IS_ENEMY | KING_IS_EMPTY)
      | self.gen_queen_moves(self.queens() & self.enemy, ALLY_IS_ENEMY | KING_IS_EMPTY)
      | self.gen_king_default_moves(self.kings() & self.enemy, ALLY_IS_ENEMY);
  }

  fn update_status(&mut self) {
    let checked = self.king_danger & self.ally_king > 0;
    let no_moves = self.get_piece_moves(self.ally) == 0;

    let winner = if_mask(self.white_turn_mask, BLACK_WON, WHITE_WON);

    let checkmate = mask_from_bool(checked && no_moves);
    let stalemate = mask_from_bool(!checked && no_moves);
    let fifty_move = mask_from_bool(self.half_clock > 100);

    let king_vs_king = self.white.only_king() && self.black.only_king();
    let king_bishop_vs_king = (self.white.only_king_and_bishop() && self.black.only_king())
      || (self.white.only_king() && self.black.only_king_and_bishop());
    let king_knight_vs_king = (self.white.only_king_and_knight() && self.black.only_king())
      || (self.white.only_king() && self.black.only_king_and_knight());

    let mut king_bishop_vs_king_bishop =
      self.white.only_king_and_bishop() && self.black.only_king_and_bishop();

    let same_color =
      self.white.bishops.trailing_zeros() % 2 == self.black.bishops.trailing_zeros() % 2;
    king_bishop_vs_king_bishop &= same_color;

    let insufficient_material = mask_from_bool(
      king_vs_king || king_bishop_vs_king | king_knight_vs_king || king_bishop_vs_king_bishop,
    );

    self.status = (checkmate & winner) | (insufficient_material | fifty_move | stalemate & DRAW);
  }

  fn update_clocks(&mut self, from_mask: u64, move_mask: u64) {
    let pawn_moved =
      move_mask & self.gen_pawn_default_move(from_mask) > 0 || self.en_passant_mask > 0;
    let capture_or_pawn = mask_from_bool(pawn_moved || move_mask & self.enemy > 0);
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
      & if_mask(
        self.white_turn_mask,
        move_mask.move_down_mask(1),
        move_mask.move_up_mask(1),
      );

    self.white.remove_piece(en_passanted_pawn);
    self.black.remove_piece(en_passanted_pawn);
  }

  fn handle_pawn_advance(&mut self, from_mask: u64, move_mask: u64) {
    let pawn_advance_move = self.gen_pawn_advance_move(from_mask & self.pawns());
    let pawn_advanced = mask_from_bool(move_mask & pawn_advance_move > 0);
    self.advance_mask &= !(pawn_advanced & from_mask);
    self.en_passant_mask = if_bool(
      move_mask > 0,
      if_mask(
        self.white_turn_mask,
        pawn_advance_move.move_down_mask(1),
        pawn_advance_move.move_up_mask(1),
      ),
      self.en_passant_mask,
    );
  }

  fn handle_castling(&mut self, from_mask: u64, move_mask: u64) {
    let king_move = self.gen_king_default_moves(from_mask, NONE);
    let rook_move = self.gen_rook_moves(from_mask, NONE);
    let revoke_castling_move = move_mask & (king_move | rook_move);
    self.castling_mask &= !revoke_castling_move;

    let long_castled = move_mask & self.gen_king_long_castle_moves(from_mask);
    let short_castled = move_mask & self.gen_king_short_castle_moves(from_mask);

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

  pub fn get_piece_moves(&self, at_mask: u64) -> u64 {
    let pawn_default_move = self.gen_pawn_default_move(at_mask & self.pawns());
    let pawn_advance_move = self.gen_pawn_advance_move(at_mask & self.pawns());
    let pawn_capturing_move = self.gen_pawn_capturing_moves(at_mask & self.pawns(), NONE);
    let pawn_moves = pawn_default_move | pawn_advance_move | pawn_capturing_move;
    let knight_moves = self.gen_knight_moves(at_mask & self.knights(), NONE);
    let bishop_moves = self.gen_bishop_moves(at_mask & self.bishops(), NONE);
    let rook_moves = self.gen_rook_moves(at_mask & self.rooks(), NONE);
    let queen_moves = self.gen_queen_moves(at_mask & self.queens(), NONE);
    let king_default_move =
      self.gen_king_default_moves(at_mask & self.kings(), NONE) & !self.king_danger;
    let king_long_castle = self.gen_king_long_castle_moves(at_mask & self.kings());
    let king_short_castle = self.gen_king_short_castle_moves(at_mask & self.kings());
    let king_moves = king_default_move | king_long_castle | king_short_castle;

    let pseudo = pawn_moves | knight_moves | bishop_moves | rook_moves | queen_moves | king_moves;

    let pin_filter = self.gen_pin_filter(at_mask);
    let check_filter = self.gen_check_filter(at_mask);

    pseudo & pin_filter & check_filter
  }
}
