pub trait BoardMovement {
  fn move_left_mask(self, amount: u32) -> u64;
  fn move_up_mask(self, amount: u32) -> u64;
  fn move_right_mask(self, amount: u32) -> u64;
  fn move_down_mask(self, amount: u32) -> u64;
}

impl BoardMovement for u64 {
  fn move_left_mask(self, amount: u32) -> u64 {
    let mut result = 0;

    for i in 0..8 {
      const MASK: u64 = 0b11111111;
      let mut byte = ((self >> (i * 8)) & MASK) as u8;
      byte = if amount < 8 { byte >> amount } else { 0 };
      result |= (byte as u64) << (i * 8);
    }
    result
  }

  fn move_up_mask(self, amount: u32) -> u64 {
    if amount < 8 { self >> 8 * amount } else { 0 }
  }

  fn move_right_mask(self, amount: u32) -> u64 {
    let mut result = 0;

    for i in 0..8 {
      const MASK: u64 = 0b11111111;
      let mut byte = ((self >> (i * 8)) & MASK) as u8;
      byte = if amount < 8 { byte << amount } else { 0 };
      result |= (byte as u64) << (i * 8);
    }
    result
  }

  fn move_down_mask(self, amount: u32) -> u64 {
    if amount < 8 { self << 8 * amount } else { 0 }
  }
}
