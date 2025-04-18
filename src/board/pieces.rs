use super::util_fns::if_bool;

// rank 8 file h is bit 0, rank 1 file a is bit 63 (so top to bottom, right to left)
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
    Pieces {
      pawns: 0,
      knights: 0,
      bishops: 0,
      rooks: 0,
      queens: 0,
      king: 0,
    }
  }

  pub fn white() -> Pieces {
    Pieces {
      pawns: 0x00_FF_00_00_00_00_00_00,
      knights: 0x42_00_00_00_00_00_00_00,
      bishops: 0x24_00_00_00_00_00_00_00,
      rooks: 0x81_00_00_00_00_00_00_00,
      queens: 0x10_00_00_00_00_00_00_00,
      king: 0x08_00_00_00_00_00_00_00,
    }
  }

  pub fn black() -> Pieces {
    Pieces {
      pawns: 0xFF_00,
      knights: 0x42,
      bishops: 0x24,
      rooks: 0x81,
      queens: 0x10,
      king: 0x08,
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


#[cfg(test)]
mod tests {
  mod movement {
    use crate::board::pieces::Pieces;

    #[test]
    fn valid_move() {
      let mut pieces = Pieces::white();
      let from = 0x00_80_00_00_00_00_00_00;
      let to = 0x00_00_80_00_00_00_00_00;
      pieces.move_piece(from, to);
      assert_eq!(pieces.pawns, 0x00_7F_80_00_00_00_00_00);
      assert_eq!(pieces.pieces_concat(), 0xFF_7F_80_00_00_00_00_00);
    }

    #[test]
    fn zero_from() {
      let mut pieces = Pieces::white();
      let from = 0;
      let to = 0x00_00_80_00_00_00_00_00;
      pieces.move_piece(from, to);
      assert_eq!(pieces.pawns, 0x00_FF_00_00_00_00_00_00);
      assert_eq!(pieces.pieces_concat(), 0xFF_FF_00_00_00_00_00_00);
    }

    #[test]
    fn empty_from() {
      let mut pieces = Pieces::white();
      let from = 1;
      let to = 0x00_00_80_00_00_00_00_00;
      pieces.move_piece(from, to);
      assert_eq!(pieces.pawns, 0x00_FF_00_00_00_00_00_00);
      assert_eq!(pieces.pieces_concat(), 0xFF_FF_00_00_00_00_00_00);
    }

    #[test]
    fn zero_to() {
      let mut pieces = Pieces::white();
      let from = 0x00_80_00_00_00_00_00_00;
      let to = 0;
      pieces.move_piece(from, to);
      assert_eq!(pieces.pawns, 0x00_FF_00_00_00_00_00_00);
      assert_eq!(pieces.pieces_concat(), 0xFF_FF_00_00_00_00_00_00);
    }

    #[test]
    fn occupied_to() {
      let mut pieces = Pieces::white();
      let from = 0x00_80_00_00_00_00_00_00;
      let to = 0x80_00_00_00_00_00_00_00;
      pieces.move_piece(from, to);
      assert_eq!(pieces.pawns, 0x00_FF_00_00_00_00_00_00);
      assert_eq!(pieces.pieces_concat(), 0xFF_FF_00_00_00_00_00_00);
    }

    #[test]
    fn from_eq_to() {
      let mut pieces = Pieces::white();
      let from = 0x00_80_00_00_00_00_00_00;
      let to = 0x00_80_00_00_00_00_00_00;
      pieces.move_piece(from, to);
      assert_eq!(pieces.pawns, 0x00_FF_00_00_00_00_00_00);
      assert_eq!(pieces.pieces_concat(), 0xFF_FF_00_00_00_00_00_00);
    }
  }
}