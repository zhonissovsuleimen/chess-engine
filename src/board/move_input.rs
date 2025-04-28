use super::util_fns::mask_from_bool;

pub const NONE: u64 = 0;
pub const KNIGHT: u64 = 1;
pub const BISHOP: u64 = 2;
pub const ROOK: u64 = 4;
pub const QUEEN: u64 = 8;

pub struct MoveInput {
  pub from: u64,
  pub to: u64,
  pub promotion: u64,
}

impl MoveInput {
  pub fn default(from: u64, to: u64) -> MoveInput {
    MoveInput {
      from: from,
      to: to,
      promotion: 0,
    }
  }

  pub fn from_id(from: usize, to: usize) -> MoveInput {
    let valid = mask_from_bool(from < 64 && to < 64 && from != to);
    MoveInput {
      from: valid & 1 << from,
      to: valid & 1 << to,
      promotion: 0,
    }
  }

  pub fn with_promotion(from: u64, to: u64, promotion: u64) -> MoveInput {
    let valid = mask_from_bool(
      promotion == KNIGHT || promotion == BISHOP || promotion == ROOK || promotion == QUEEN,
    );
    MoveInput {
      from: valid & from,
      to: valid & to,
      promotion: valid & promotion,
    }
  }
}
