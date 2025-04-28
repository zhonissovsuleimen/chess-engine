#[derive(Default)]
pub struct CachedPieceMoves {
  pub pawn_default: u64,
  pub pawn_advance: u64,
  pub pawn_capture: u64,
  pub pawn_promote: u64,
  pub knight: u64,
  pub bishop: u64,
  pub rook: u64,
  pub queen: u64,
  pub king_default: u64,
  pub king_short_castle: u64,
  pub king_long_castle: u64,
  pub capturing: u64,
  pub status: u64,
}

impl CachedPieceMoves {
  pub fn all(&self) -> u64 {
    let pawn_moves = self.pawn_default | self.pawn_advance | self.pawn_capture | self.pawn_promote;
    let knight_moves = self.knight;
    let bishop_moves = self.bishop;
    let rook_moves = self.rook;
    let queen_moves = self.queen;
    let king_moves = self.king_default | self.king_long_castle | self.king_short_castle;

    pawn_moves | knight_moves | bishop_moves | rook_moves | queen_moves | king_moves
  }
}