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
}