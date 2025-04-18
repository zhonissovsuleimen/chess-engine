#[derive(Default)]
pub struct CachedPieceMoves {
  pub pawn_default: u64,
  pub pawn_advance: u64,
  pub pawn_capturing: u64,
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
    let pawn_default_move = self.pawn_default;
    let pawn_advance_move = self.pawn_advance;
    let pawn_capturing_move = self.pawn_capturing;
    let pawn_moves = pawn_default_move | pawn_advance_move | pawn_capturing_move;
    let knight_moves = self.knight;
    let bishop_moves = self.bishop;
    let rook_moves = self.rook;
    let queen_moves = self.queen;
    let king_default_move = self.king_default;
    let king_long_castle = self.king_long_castle;
    let king_short_castle = self.king_short_castle;
    let king_moves = king_default_move | king_long_castle | king_short_castle;

    pawn_moves | knight_moves | bishop_moves | rook_moves | queen_moves | king_moves
  }
}