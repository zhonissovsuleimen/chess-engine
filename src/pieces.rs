// rank 8 file a is bit 0, rank 1 file h is bit 63
pub struct Pieces {
  pub pawns: u64,
  pub knights: u64,
  pub bishops: u64,
  pub rooks: u64,
  pub queens: u64,
  pub king: u64
}

impl Pieces {
  pub fn empty() -> Pieces {
    Pieces { pawns: 0, knights: 0, bishops: 0, rooks: 0, queens: 0, king: 0 }
  }

  pub fn white() -> Pieces {
    Pieces { pawns: 71776119061217280, knights: 4755801206503243776, bishops: 2594073385365405696, rooks: 9295429630892703744, queens: 576460752303423488, king: 1152921504606846976 }
  }

  pub fn black() -> Pieces {
    Pieces { pawns: 65280, knights: 66, bishops: 36, rooks: 129, queens: 8, king: 16 }
  }

  pub fn pieces_as_array(&self) -> [&u64; 6] {
    return [&self.pawns, &self.knights, &self.bishops, &self.rooks, &self.queens, &self.king];
  }

  pub fn pieces_as_mut_array(&mut self) -> [&mut u64; 6] {
    return [&mut self.pawns, &mut self.knights, &mut self.bishops, &mut self.rooks, &mut self.queens, &mut self.king];
  }

  pub fn pieces_concat(&self) -> u64 {
    return self.pawns + self.knights + self.bishops + self.rooks + self.queens + self.king;
  }
}

//moves & state
impl Pieces {
  pub fn r#move(&mut self, from_mask: u64, to_mask: u64) {
    //todo: branchless bitwise trickery
    for i in 0..6 {
      match i {
        0 if self.pawns & from_mask > 0 => {
          self.pawns -= from_mask;
          self.pawns += to_mask;
        }
        1 if self.knights & from_mask > 0 => {
          self.knights -= from_mask;
          self.knights += to_mask;
        }
        2 if self.bishops & from_mask > 0 => {
          self.bishops -= from_mask;
          self.bishops += to_mask;
        }
        3 if self.rooks & from_mask > 0 => {
          self.rooks -= from_mask;
          self.rooks += to_mask;
        }
        4 if self.queens & from_mask > 0 => {
          self.queens -= from_mask;
          self.queens += to_mask;
        }
        5 if self.king & from_mask > 0 => {
          self.king -= from_mask;
          self.king += to_mask;
        }
        _ => {}
      }
    }
  }

  pub fn is_empty(&self, at_mask: u64) -> bool {
    self.pieces_concat() & at_mask == 1
  }
}
