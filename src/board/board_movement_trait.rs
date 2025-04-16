use super::util_fns::mask_from_bool;

pub trait BoardMovement {
  fn move_left_mask(self, amount: u32) -> u64;
  fn move_up_mask(self, amount: u32) -> u64;
  fn move_right_mask(self, amount: u32) -> u64;
  fn move_down_mask(self, amount: u32) -> u64;
}

impl BoardMovement for u64 {
  fn move_left_mask(self, amount: u32) -> u64 {
    let shift = amount & 7;
    let byte_shift_mask = (0xFF_u8 >> shift) as u64;
    let per_byte_mask = 0x0101010101010101u64.wrapping_mul(byte_shift_mask);
    let mask = mask_from_bool(amount < 8);
    (self >> shift) & (per_byte_mask & mask)
  }

  fn move_up_mask(self, amount: u32) -> u64 {
    let shift = 8 * (amount & 7);
    let mask = mask_from_bool(amount < 8);
    (self >> shift) & mask
  }

  fn move_right_mask(self, amount: u32) -> u64 {
    let shift = amount & 7;
    let byte_shift_mask = (0xFF_u8 << shift) as u64;
    let per_byte_mask = 0x0101010101010101u64.wrapping_mul(byte_shift_mask);
    let mask = mask_from_bool(amount < 8);
    (self << shift) & per_byte_mask & mask
  }

  fn move_down_mask(self, amount: u32) -> u64 {
    let shift = 8 * (amount & 7);
    let mask = ((amount < 8) as u64).wrapping_neg();
    (self << shift) & mask
  }
}
