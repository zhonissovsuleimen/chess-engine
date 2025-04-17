use super::util_fns::if_bool;

// rank 8 file a is bit 0, rank 1 file h is bit 63
#[derive(Default)]
pub struct Pieces {
  pub pawns: u64,
  pub knights: u64,
  pub bishops: u64,
  pub rooks: u64,
  pub queens: u64,
  pub king: u64,
}

//constructors
impl Pieces {
  pub fn empty() -> Pieces {
    Pieces::default()
  }

  pub fn white() -> Pieces {
    Pieces {
      pawns: 71776119061217280,
      knights: 4755801206503243776,
      bishops: 2594073385365405696,
      rooks: 9295429630892703744,
      queens: 576460752303423488,
      king: 1152921504606846976,
    }
  }

  pub fn black() -> Pieces {
    Pieces {
      pawns: 65280,
      knights: 66,
      bishops: 36,
      rooks: 129,
      queens: 8,
      king: 16,
    }
  }
}

//auxilary
impl Pieces {
  pub fn pieces_as_mut_array(&mut self) -> [&mut u64; 6] {
    [
      &mut self.pawns,
      &mut self.knights,
      &mut self.bishops,
      &mut self.rooks,
      &mut self.queens,
      &mut self.king,
    ]
  }

  pub fn pieces_concat(&self) -> u64 {
    self.pawns | self.knights | self.bishops | self.rooks | self.queens | self.king
  }
}

//moves & state
impl Pieces {
  pub fn move_piece(&mut self, from_mask: u64, to_mask: u64) {
    let pieces = self.pieces_concat();
    let valid = from_mask & pieces > 0 && !pieces & to_mask > 0;

    self.pawns = if_bool(
      valid && self.is_pawn(from_mask),
      (self.pawns & !from_mask) | to_mask,
      self.pawns,
    );
    self.knights = if_bool(
      valid && self.is_knight(from_mask),
      (self.knights & !from_mask) | to_mask,
      self.knights,
    );
    self.bishops = if_bool(
      valid && self.is_bishop(from_mask),
      (self.bishops & !from_mask) | to_mask,
      self.bishops,
    );
    self.rooks = if_bool(
      valid && self.is_rook(from_mask),
      (self.rooks & !from_mask) | to_mask,
      self.rooks,
    );
    self.queens = if_bool(
      valid && self.is_queen(from_mask),
      (self.queens & !from_mask) | to_mask,
      self.queens,
    );
    self.king = if_bool(
      valid && self.is_king(from_mask),
      (self.king & !from_mask) | to_mask,
      self.king,
    );
  }

  pub fn remove_piece(&mut self, at_mask: u64) {
    for piece in self.pieces_as_mut_array() {
      *piece &= !at_mask;
    }
  }

  pub fn is_pawn(&self, at_mask: u64) -> bool {
    self.pawns & at_mask > 0
  }

  pub fn is_knight(&self, at_mask: u64) -> bool {
    self.knights & at_mask > 0
  }

  pub fn is_bishop(&self, at_mask: u64) -> bool {
    self.bishops & at_mask > 0
  }

  pub fn is_rook(&self, at_mask: u64) -> bool {
    self.rooks & at_mask > 0
  }
  pub fn is_queen(&self, at_mask: u64) -> bool {
    self.queens & at_mask > 0
  }

  pub fn is_king(&self, at_mask: u64) -> bool {
    self.king & at_mask > 0
  }

  pub fn only_king(&self) -> bool {
    self.pieces_concat() == self.king
  }

  pub fn only_king_and_bishop(&self) -> bool {
    let one_bishop = self.bishops.count_ones() == 1;
    one_bishop && (self.pieces_concat() == self.king | self.bishops)
  }

  pub fn only_king_and_knight(&self) -> bool {
    let one_knight = self.knights.count_ones() == 1;
    one_knight && (self.pieces_concat() == self.king | self.knights)
  }
}
