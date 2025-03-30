// rank 8 file a is bit 0, rank 1 file h is bit 63
pub struct Pieces {
  pub pawns: u64,
  pub knights: u64,
  pub bishops: u64,
  pub rooks: u64,
  pub queens: u64,
  pub king: u64,

  pub pawns_advance: u64,
}

//constructors
impl Pieces {
  pub fn empty() -> Pieces {
    Pieces {
      pawns: 0,
      knights: 0,
      bishops: 0,
      rooks: 0,
      queens: 0,
      king: 0,
      pawns_advance: 0,
    }
  }

  pub fn white() -> Pieces {
    Pieces {
      pawns: 71776119061217280,
      knights: 4755801206503243776,
      bishops: 2594073385365405696,
      rooks: 9295429630892703744,
      queens: 576460752303423488,
      king: 1152921504606846976,
      pawns_advance: 71776119061217280,
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
      pawns_advance: 65280,
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
    return self.pawns + self.knights + self.bishops + self.rooks + self.queens + self.king;
  }
}

//moves & state
impl Pieces {
  pub fn r#move(&mut self, from_mask: u64, to_mask: u64) {
    //todo: branchless bitwise trickery
    if self.is_pawn(from_mask) {
      self.pawns -= from_mask;
      self.pawns += to_mask;
    } else if self.is_knight(from_mask) {
      self.knights -= from_mask;
      self.knights += to_mask;
    } else if self.is_bishop(from_mask) {
      self.bishops -= from_mask;
      self.bishops += to_mask;
    } else if self.is_rook(from_mask) {
      self.rooks -= from_mask;
      self.rooks += to_mask;
    } else if self.is_queen(from_mask) {
      self.queens -= from_mask;
      self.queens += to_mask;
    } else if self.is_king(from_mask) {
      self.king -= from_mask;
      self.king += to_mask;
    }
  }

  pub fn remove_advance(&mut self, at_mask: u64) {
    self.pawns_advance &= !at_mask;
  }

  pub fn is_empty(&self, at_mask: u64) -> bool {
    self.pieces_concat() & at_mask == 0
  }

  pub fn can_advance(&self, at_mask: u64) -> bool {
    return (self.pawns & self.pawns_advance & at_mask) > 0;
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
