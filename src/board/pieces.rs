use super::util_fns::branchless_if;

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
  pub fn pieces_as_array(&self) -> [&u64; 6] {
    return [
      &self.pawns,
      &self.knights,
      &self.bishops,
      &self.rooks,
      &self.queens,
      &self.king,
    ];
  }

  pub fn pieces_as_mut_array(&mut self) -> [&mut u64; 6] {
    return [
      &mut self.pawns,
      &mut self.knights,
      &mut self.bishops,
      &mut self.rooks,
      &mut self.queens,
      &mut self.king,
    ];
  }

  pub fn pieces_concat(&self) -> u64 {
    return self.pawns | self.knights | self.bishops | self.rooks | self.queens | self.king;
  }

  pub fn get_value(&self) -> i32 {
    let mut total = 0;
    let lookup = [1, 3, 3, 5, 9, 1000];

    for (i, piece) in self.pieces_as_array().iter().enumerate() {
      total += lookup[i] * piece.count_zeros();
    }

    total as i32
  }
}

//moves & state
impl Pieces {
  pub fn move_piece(&mut self, from_mask: u64, to_mask: u64) {
    let valid = from_mask > 0 && to_mask > 0;
    self.pawns = branchless_if(valid && self.pawns & from_mask > 0, (self.pawns & !from_mask) | to_mask, self.pawns);
    self.knights = branchless_if(valid && self.knights & from_mask > 0, (self.knights & !from_mask) | to_mask, self.knights);
    self.bishops = branchless_if(valid && self.bishops & from_mask > 0, (self.bishops & !from_mask) | to_mask, self.bishops);
    self.rooks = branchless_if(valid && self.rooks & from_mask > 0, (self.rooks & !from_mask) | to_mask, self.rooks);
    self.queens = branchless_if(valid && self.queens & from_mask > 0, (self.queens & !from_mask) | to_mask, self.queens);
    self.king = branchless_if(valid && self.king & from_mask > 0, (self.king & !from_mask) | to_mask, self.king);
  }

  pub fn remove_piece(&mut self, at_mask: u64) {
    for piece in self.pieces_as_mut_array() {
      *piece &= !at_mask;
    }
  }

  pub fn is_empty(&self, at_mask: u64) -> bool {
    self.pieces_concat() & at_mask == 0
  }

  pub fn is_pawn(&self, at_mask: u64) -> bool {
    return self.pawns & at_mask > 0;
  }

  pub fn is_knight(&self, at_mask: u64) -> bool {
    return self.knights & at_mask > 0;
  }

  pub fn is_bishop(&self, at_mask: u64) -> bool {
    return self.bishops & at_mask > 0;
  }

  pub fn is_rook(&self, at_mask: u64) -> bool {
    return self.rooks & at_mask > 0;
  }
  pub fn is_queen(&self, at_mask: u64) -> bool {
    return self.queens & at_mask > 0;
  }

  pub fn is_king(&self, at_mask: u64) -> bool {
    return self.king & at_mask > 0;
  }
}