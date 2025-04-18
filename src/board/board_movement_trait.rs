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
    let byte_shift_mask = (0xFF_u8 << shift) as u64;
    let per_byte_mask = 0x0101010101010101u64.wrapping_mul(byte_shift_mask);
    let mask = mask_from_bool(amount < 8);
    (self << shift) & (per_byte_mask & mask)
  }

  fn move_up_mask(self, amount: u32) -> u64 {
    let shift = 8 * (amount & 7);
    let mask = mask_from_bool(amount < 8);
    (self >> shift) & mask
  }

  fn move_right_mask(self, amount: u32) -> u64 {
    let shift = amount & 7;
    let byte_shift_mask = (0xFF_u8 >> shift) as u64;
    let per_byte_mask = 0x0101010101010101u64.wrapping_mul(byte_shift_mask);
    let mask = mask_from_bool(amount < 8);
    (self >> shift) & per_byte_mask & mask
  }

  fn move_down_mask(self, amount: u32) -> u64 {
    let shift = 8 * (amount & 7);
    let mask = ((amount < 8) as u64).wrapping_neg();
    (self << shift) & mask
  }
}

#[cfg(test)]
mod tests {
  use super::BoardMovement;

  mod right {
    use super::BoardMovement;
    #[test]
    fn bit_move_by_zero() {
      let bit0 = 0x80;
      assert_eq!(bit0.move_right_mask(0), bit0);
      let bit1 = 0x80_00;
      assert_eq!(bit1.move_right_mask(0), bit1);
      let bit2 = 0x80_00_00;
      assert_eq!(bit2.move_right_mask(0), bit2);
      let bit3 = 0x80_00_00_00;
      assert_eq!(bit3.move_right_mask(0), bit3);
      let bit4 = 0x80_00_00_00_00;
      assert_eq!(bit4.move_right_mask(0), bit4);
      let bit5 = 0x80_00_00_00_00_00;
      assert_eq!(bit5.move_right_mask(0), bit5);
      let bit6 = 0x80_00_00_00_00_00_00;
      assert_eq!(bit6.move_right_mask(0), bit6);
      let bit7 = 0x80_00_00_00_00_00_00_00;
      assert_eq!(bit7.move_right_mask(0), bit7);
    }

    #[test]
    fn bit_move_by_one() {
      let bit0 = 0x80;
      assert_eq!(bit0.move_right_mask(1), 0x40);
      let bit1 = 0x80_00;
      assert_eq!(bit1.move_right_mask(1), 0x40_00);
      let bit2 = 0x80_00_00;
      assert_eq!(bit2.move_right_mask(1), 0x40_00_00);
      let bit3 = 0x80_00_00_00;
      assert_eq!(bit3.move_right_mask(1), 0x40_00_00_00);
      let bit4 = 0x80_00_00_00_00;
      assert_eq!(bit4.move_right_mask(1), 0x40_00_00_00_00);
      let bit5 = 0x80_00_00_00_00_00;
      assert_eq!(bit5.move_right_mask(1), 0x40_00_00_00_00_00);
      let bit6 = 0x80_00_00_00_00_00_00;
      assert_eq!(bit6.move_right_mask(1), 0x40_00_00_00_00_00_00);
      let bit7 = 0x80_00_00_00_00_00_00_00;
      assert_eq!(bit7.move_right_mask(1), 0x40_00_00_00_00_00_00_00);
    }

    #[test]
    fn bit_move_by_eight() {
      let bit0 = 0x80;
      assert_eq!(bit0.move_right_mask(8), 0);
      let bit1 = 0x80_00;
      assert_eq!(bit1.move_right_mask(8), 0);
      let bit2 = 0x80_00_00;
      assert_eq!(bit2.move_right_mask(8), 0);
      let bit3 = 0x80_00_00_00;
      assert_eq!(bit3.move_right_mask(8), 0);
      let bit4 = 0x80_00_00_00_00;
      assert_eq!(bit4.move_right_mask(8), 0);
      let bit5 = 0x80_00_00_00_00_00;
      assert_eq!(bit5.move_right_mask(8), 0);
      let bit6 = 0x80_00_00_00_00_00_00;
      assert_eq!(bit6.move_right_mask(8), 0);
      let bit7 = 0x80_00_00_00_00_00_00_00;
      assert_eq!(bit7.move_right_mask(8), 0);
    }

    #[test]
    fn bit_move_to_out_of_bounds() {
      let bit0 = 0x80;
      assert_eq!(bit0.move_right_mask(8), 0);
      let bit1 = 0x40_00;
      assert_eq!(bit1.move_right_mask(7), 0);
      let bit2 = 0x20_00_00;
      assert_eq!(bit2.move_right_mask(6), 0);
      let bit3 = 0x10_00_00_00;
      assert_eq!(bit3.move_right_mask(5), 0);
      let bit4 = 0x08_00_00_00_00;
      assert_eq!(bit4.move_right_mask(4), 0);
      let bit5 = 0x04_00_00_00_00_00;
      assert_eq!(bit5.move_right_mask(3), 0);
      let bit6 = 0x02_00_00_00_00_00_00;
      assert_eq!(bit6.move_right_mask(2), 0);
      let bit7 = 0x01_00_00_00_00_00_00_00;
      assert_eq!(bit7.move_right_mask(1), 0);
    }

    #[test]
    fn hor_byte_move_by_zero() {
      let byte0 = 0xFF;
      assert_eq!(byte0.move_right_mask(0), byte0);
      let byte1 = 0xFF_00;
      assert_eq!(byte1.move_right_mask(0), byte1);
      let byte2 = 0xFF_00_00;
      assert_eq!(byte2.move_right_mask(0), byte2);
      let byte3 = 0xFF_00_00_00;
      assert_eq!(byte3.move_right_mask(0), byte3);
      let byte4 = 0xFF_00_00_00_00;
      assert_eq!(byte4.move_right_mask(0), byte4);
      let byte5 = 0xFF_00_00_00_00_00;
      assert_eq!(byte5.move_right_mask(0), byte5);
      let byte6 = 0xFF_00_00_00_00_00_00;
      assert_eq!(byte6.move_right_mask(0), byte6);
      let byte7 = 0xFF_00_00_00_00_00_00_00;
      assert_eq!(byte7.move_right_mask(0), byte7);
    }

    #[test]
    fn hor_byte_move_by_one() {
      let byte0 = 0xFF;
      assert_eq!(byte0.move_right_mask(1), 0x7F);
      let byte1 = 0xFF_00;
      assert_eq!(byte1.move_right_mask(1), 0x7F_00);
      let byte2 = 0xFF_00_00;
      assert_eq!(byte2.move_right_mask(1), 0x7F_00_00);
      let byte3 = 0xFF_00_00_00;
      assert_eq!(byte3.move_right_mask(1), 0x7F_00_00_00);
      let byte4 = 0xFF_00_00_00_00;
      assert_eq!(byte4.move_right_mask(1), 0x7F_00_00_00_00);
      let byte5 = 0xFF_00_00_00_00_00;
      assert_eq!(byte5.move_right_mask(1), 0x7F_00_00_00_00_00);
      let byte6 = 0xFF_00_00_00_00_00_00;
      assert_eq!(byte6.move_right_mask(1), 0x7F_00_00_00_00_00_00);
      let byte7 = 0xFF_00_00_00_00_00_00_00;
      assert_eq!(byte7.move_right_mask(1), 0x7F_00_00_00_00_00_00_00);
    }

    #[test]
    fn hor_byte_move_by_eight() {
      let byte0 = 0xFF;
      assert_eq!(byte0.move_right_mask(8), 0);
      let byte1 = 0xFF_00;
      assert_eq!(byte1.move_right_mask(8), 0);
      let byte2 = 0xFF_00_00;
      assert_eq!(byte2.move_right_mask(8), 0);
      let byte3 = 0xFF_00_00_00;
      assert_eq!(byte3.move_right_mask(8), 0);
      let byte4 = 0xFF_00_00_00_00;
      assert_eq!(byte4.move_right_mask(8), 0);
      let byte5 = 0xFF_00_00_00_00_00;
      assert_eq!(byte5.move_right_mask(8), 0);
      let byte6 = 0xFF_00_00_00_00_00_00;
      assert_eq!(byte6.move_right_mask(8), 0);
      let byte7 = 0xFF_00_00_00_00_00_00_00;
      assert_eq!(byte7.move_right_mask(8), 0);
    }

    #[test]
    fn hor_byte_move_to_out_of_bounds() {
      let byte0 = 0xFF;
      assert_eq!(byte0.move_right_mask(8), 0);
      let byte1 = 0x7F_00;
      assert_eq!(byte1.move_right_mask(7), 0);
      let byte2 = 0x3F_00_00;
      assert_eq!(byte2.move_right_mask(6), 0);
      let byte3 = 0x1F_00_00_00;
      assert_eq!(byte3.move_right_mask(5), 0);
      let byte4 = 0x0F_00_00_00_00;
      assert_eq!(byte4.move_right_mask(4), 0);
      let byte5 = 0x07_00_00_00_00_00;
      assert_eq!(byte5.move_right_mask(3), 0);
      let byte6 = 0x03_00_00_00_00_00_00;
      assert_eq!(byte6.move_right_mask(2), 0);
      let byte7 = 0x01_00_00_00_00_00_00_00;
      assert_eq!(byte7.move_right_mask(1), 0);
    }
  }

  mod left {
    use super::BoardMovement;

    #[test]
    fn bit_move_by_zero() {
      let bit0 = 0x01;
      assert_eq!(bit0.move_left_mask(0), bit0);
      let bit1 = 0x01_00;
      assert_eq!(bit1.move_left_mask(0), bit1);
      let bit2 = 0x01_00_00;
      assert_eq!(bit2.move_left_mask(0), bit2);
      let bit3 = 0x01_00_00_00;
      assert_eq!(bit3.move_left_mask(0), bit3);
      let bit4 = 0x01_00_00_00_00;
      assert_eq!(bit4.move_left_mask(0), bit4);
      let bit5 = 0x01_00_00_00_00_00;
      assert_eq!(bit5.move_left_mask(0), bit5);
      let bit6 = 0x01_00_00_00_00_00_00;
      assert_eq!(bit6.move_left_mask(0), bit6);
      let bit7 = 0x01_00_00_00_00_00_00_00;
      assert_eq!(bit7.move_left_mask(0), bit7);
    }

    #[test]
    fn bit_move_by_one() {
      let bit0 = 0x01;
      assert_eq!(bit0.move_left_mask(1), 0x02);
      let bit1 = 0x01_00;
      assert_eq!(bit1.move_left_mask(1), 0x02_00);
      let bit2 = 0x01_00_00;
      assert_eq!(bit2.move_left_mask(1), 0x02_00_00);
      let bit3 = 0x01_00_00_00;
      assert_eq!(bit3.move_left_mask(1), 0x02_00_00_00);
      let bit4 = 0x01_00_00_00_00;
      assert_eq!(bit4.move_left_mask(1), 0x02_00_00_00_00);
      let bit5 = 0x01_00_00_00_00_00;
      assert_eq!(bit5.move_left_mask(1), 0x02_00_00_00_00_00);
      let bit6 = 0x01_00_00_00_00_00_00;
      assert_eq!(bit6.move_left_mask(1), 0x02_00_00_00_00_00_00);
      let bit7 = 0x01_00_00_00_00_00_00_00;
      assert_eq!(bit7.move_left_mask(1), 0x02_00_00_00_00_00_00_00);
    }

    #[test]
    fn bit_move_by_eight() {
      let bit0 = 0x01;
      assert_eq!(bit0.move_left_mask(8), 0);
      let bit1 = 0x01_00;
      assert_eq!(bit1.move_left_mask(8), 0);
      let bit2 = 0x01_00_00;
      assert_eq!(bit2.move_left_mask(8), 0);
      let bit3 = 0x01_00_00_00;
      assert_eq!(bit3.move_left_mask(8), 0);
      let bit4 = 0x01_00_00_00_00;
      assert_eq!(bit4.move_left_mask(8), 0);
      let bit5 = 0x01_00_00_00_00_00;
      assert_eq!(bit5.move_left_mask(8), 0);
      let bit6 = 0x01_00_00_00_00_00_00;
      assert_eq!(bit6.move_left_mask(8), 0);
      let bit7 = 0x01_00_00_00_00_00_00_00;
      assert_eq!(bit7.move_left_mask(8), 0);
    }

    #[test]
    fn bit_move_to_out_of_bounds() {
      let bit0 = 0x01;
      assert_eq!(bit0.move_left_mask(8), 0);
      let bit1 = 0x02_00;
      assert_eq!(bit1.move_left_mask(7), 0);
      let bit2 = 0x04_00_00;
      assert_eq!(bit2.move_left_mask(6), 0);
      let bit3 = 0x08_00_00_00;
      assert_eq!(bit3.move_left_mask(5), 0);
      let bit4 = 0x10_00_00_00_00;
      assert_eq!(bit4.move_left_mask(4), 0);
      let bit5 = 0x20_00_00_00_00_00;
      assert_eq!(bit5.move_left_mask(3), 0);
      let bit6 = 0x40_00_00_00_00_00_00;
      assert_eq!(bit6.move_left_mask(2), 0);
      let bit7 = 0x80_00_00_00_00_00_00_00;
      assert_eq!(bit7.move_left_mask(1), 0);
    }

    #[test]
    fn hor_byte_move_by_zero() {
      let byte0 = 0xFF;
      assert_eq!(byte0.move_left_mask(0), byte0);
      let byte1 = 0xFF_00;
      assert_eq!(byte1.move_left_mask(0), byte1);
      let byte2 = 0xFF_00_00;
      assert_eq!(byte2.move_left_mask(0), byte2);
      let byte3 = 0xFF_00_00_00;
      assert_eq!(byte3.move_left_mask(0), byte3);
      let byte4 = 0xFF_00_00_00_00;
      assert_eq!(byte4.move_left_mask(0), byte4);
      let byte5 = 0xFF_00_00_00_00_00;
      assert_eq!(byte5.move_left_mask(0), byte5);
      let byte6 = 0xFF_00_00_00_00_00_00;
      assert_eq!(byte6.move_left_mask(0), byte6);
      let byte7 = 0xFF_00_00_00_00_00_00_00;
      assert_eq!(byte7.move_left_mask(0), byte7);
    }

    #[test]
    fn hor_byte_move_by_one() {
      let byte0 = 0xFF;
      assert_eq!(byte0.move_left_mask(1), 0xFE);
      let byte1 = 0xFF_00;
      assert_eq!(byte1.move_left_mask(1), 0xFE_00);
      let byte2 = 0xFF_00_00;
      assert_eq!(byte2.move_left_mask(1), 0xFE_00_00);
      let byte3 = 0xFF_00_00_00;
      assert_eq!(byte3.move_left_mask(1), 0xFE_00_00_00);
      let byte4 = 0xFF_00_00_00_00;
      assert_eq!(byte4.move_left_mask(1), 0xFE_00_00_00_00);
      let byte5 = 0xFF_00_00_00_00_00;
      assert_eq!(byte5.move_left_mask(1), 0xFE_00_00_00_00_00);
      let byte6 = 0xFF_00_00_00_00_00_00;
      assert_eq!(byte6.move_left_mask(1), 0xFE_00_00_00_00_00_00);
      let byte7 = 0xFF_00_00_00_00_00_00_00;
      assert_eq!(byte7.move_left_mask(1), 0xFE_00_00_00_00_00_00_00);
    }

    #[test]
    fn hor_byte_move_by_eight() {
      let byte0 = 0xFF;
      assert_eq!(byte0.move_left_mask(8), 0);
      let byte1 = 0xFF_00;
      assert_eq!(byte1.move_left_mask(8), 0);
      let byte2 = 0xFF_00_00;
      assert_eq!(byte2.move_left_mask(8), 0);
      let byte3 = 0xFF_00_00_00;
      assert_eq!(byte3.move_left_mask(8), 0);
      let byte4 = 0xFF_00_00_00_00;
      assert_eq!(byte4.move_left_mask(8), 0);
      let byte5 = 0xFF_00_00_00_00_00;
      assert_eq!(byte5.move_left_mask(8), 0);
      let byte6 = 0xFF_00_00_00_00_00_00;
      assert_eq!(byte6.move_left_mask(8), 0);
      let byte7 = 0xFF_00_00_00_00_00_00_00;
      assert_eq!(byte7.move_left_mask(8), 0);
    }

    #[test]
    fn hor_byte_move_to_out_of_bounds() {
      let byte0 = 0xFF;
      assert_eq!(byte0.move_left_mask(8), 0);
      let byte1 = 0xFE_00;
      assert_eq!(byte1.move_left_mask(7), 0);
      let byte2 = 0xFC_00_00;
      assert_eq!(byte2.move_left_mask(6), 0);
      let byte3 = 0xF8_00_00_00;
      assert_eq!(byte3.move_left_mask(5), 0);
      let byte4 = 0xF0_00_00_00_00;
      assert_eq!(byte4.move_left_mask(4), 0);
      let byte5 = 0xE0_00_00_00_00_00;
      assert_eq!(byte5.move_left_mask(3), 0);
      let byte6 = 0xC0_00_00_00_00_00_00;
      assert_eq!(byte6.move_left_mask(2), 0);
      let byte7 = 0x80_00_00_00_00_00_00_00;
      assert_eq!(byte7.move_left_mask(1), 0);
    }
  }

  mod up {
    use super::BoardMovement;

    #[test]
    fn bit_move_by_zero() {
      let bit0 = 0x80_00_00_00_00_00_00_00;
      assert_eq!(bit0.move_up_mask(0), bit0);
      let bit1 = 0x40_00_00_00_00_00_00_00;
      assert_eq!(bit1.move_up_mask(0), bit1);
      let bit2 = 0x20_00_00_00_00_00_00_00;
      assert_eq!(bit2.move_up_mask(0), bit2);
      let bit3 = 0x10_00_00_00_00_00_00_00;
      assert_eq!(bit3.move_up_mask(0), bit3);
      let bit4 = 0x08_00_00_00_00_00_00_00;
      assert_eq!(bit4.move_up_mask(0), bit4);
      let bit5 = 0x04_00_00_00_00_00_00_00;
      assert_eq!(bit5.move_up_mask(0), bit5);
      let bit6 = 0x02_00_00_00_00_00_00_00;
      assert_eq!(bit6.move_up_mask(0), bit6);
      let bit7 = 0x01_00_00_00_00_00_00_00;
      assert_eq!(bit7.move_up_mask(0), bit7);
    }

    #[test]
    fn bit_move_by_one() {
      let bit0 = 0x80_00_00_00_00_00_00_00;
      assert_eq!(bit0.move_up_mask(1), 0x80_00_00_00_00_00_00);
      let bit1 = 0x40_00_00_00_00_00_00_00;
      assert_eq!(bit1.move_up_mask(1), 0x40_00_00_00_00_00_00);
      let bit2 = 0x20_00_00_00_00_00_00_00;
      assert_eq!(bit2.move_up_mask(1), 0x20_00_00_00_00_00_00);
      let bit3 = 0x10_00_00_00_00_00_00_00;
      assert_eq!(bit3.move_up_mask(1), 0x10_00_00_00_00_00_00);
      let bit4 = 0x08_00_00_00_00_00_00_00;
      assert_eq!(bit4.move_up_mask(1), 0x08_00_00_00_00_00_00);
      let bit5 = 0x04_00_00_00_00_00_00_00;
      assert_eq!(bit5.move_up_mask(1), 0x04_00_00_00_00_00_00);
      let bit6 = 0x02_00_00_00_00_00_00_00;
      assert_eq!(bit6.move_up_mask(1), 0x02_00_00_00_00_00_00);
      let bit7 = 0x01_00_00_00_00_00_00_00;
      assert_eq!(bit7.move_up_mask(1), 0x01_00_00_00_00_00_00);
    }

    #[test]
    fn bit_move_by_eight() {
      let bit0 = 0x80_00_00_00_00_00_00_00;
      assert_eq!(bit0.move_up_mask(8), 0);
      let bit1 = 0x40_00_00_00_00_00_00_00;
      assert_eq!(bit1.move_up_mask(8), 0);
      let bit2 = 0x20_00_00_00_00_00_00_00;
      assert_eq!(bit2.move_up_mask(8), 0);
      let bit3 = 0x10_00_00_00_00_00_00_00;
      assert_eq!(bit3.move_up_mask(8), 0);
      let bit4 = 0x08_00_00_00_00_00_00_00;
      assert_eq!(bit4.move_up_mask(8), 0);
      let bit5 = 0x04_00_00_00_00_00_00_00;
      assert_eq!(bit5.move_up_mask(8), 0);
      let bit6 = 0x02_00_00_00_00_00_00_00;
      assert_eq!(bit6.move_up_mask(8), 0);
      let bit7 = 0x01_00_00_00_00_00_00_00;
      assert_eq!(bit7.move_up_mask(8), 0);
    }

    #[test]
    fn bit_move_to_out_of_bounds() {
      let bit0 = 0x80_00_00_00_00_00_00_00;
      assert_eq!(bit0.move_up_mask(8), 0);
      let bit1 = 0x40_00_00_00_00_00_00;
      assert_eq!(bit1.move_up_mask(7), 0);
      let bit2 = 0x20_00_00_00_00_00;
      assert_eq!(bit2.move_up_mask(6), 0);
      let bit3 = 0x10_00_00_00_00;
      assert_eq!(bit3.move_up_mask(5), 0);
      let bit4 = 0x08_00_00_00;
      assert_eq!(bit4.move_up_mask(4), 0);
      let bit5 = 0x04_00_00;
      assert_eq!(bit5.move_up_mask(3), 0);
      let bit6 = 0x02_00;
      assert_eq!(bit6.move_up_mask(2), 0);
      let bit7 = 0x01;
      assert_eq!(bit7.move_up_mask(1), 0);
    }

    #[test]
    fn ver_byte_move_by_zero() {
      let byte0 = 0x80_80_80_80_80_80_80_80;
      assert_eq!(byte0.move_up_mask(0), byte0);
      let byte1 = 0x40_40_40_40_40_40_40_40;
      assert_eq!(byte1.move_up_mask(0), byte1);
      let byte2 = 0x20_20_20_20_20_20_20_20;
      assert_eq!(byte2.move_up_mask(0), byte2);
      let byte3 = 0x10_10_10_10_10_10_10_10;
      assert_eq!(byte3.move_up_mask(0), byte3);
      let byte4 = 0x08_08_08_08_08_08_08_08;
      assert_eq!(byte4.move_up_mask(0), byte4);
      let byte5 = 0x04_04_04_04_04_04_04_04;
      assert_eq!(byte5.move_up_mask(0), byte5);
      let byte6 = 0x02_02_02_02_02_02_02_02;
      assert_eq!(byte6.move_up_mask(0), byte6);
      let byte7 = 0x01_01_01_01_01_01_01_01;
      assert_eq!(byte7.move_up_mask(0), byte7);
    }

    #[test]
    fn ver_byte_move_by_one() {
      let byte0 = 0x80_80_80_80_80_80_80_80;
      assert_eq!(byte0.move_up_mask(1), 0x80_80_80_80_80_80_80);
      let byte1 = 0x40_40_40_40_40_40_40_40;
      assert_eq!(byte1.move_up_mask(1), 0x40_40_40_40_40_40_40);
      let byte2 = 0x20_20_20_20_20_20_20_20;
      assert_eq!(byte2.move_up_mask(1), 0x20_20_20_20_20_20_20);
      let byte3 = 0x10_10_10_10_10_10_10_10;
      assert_eq!(byte3.move_up_mask(1), 0x10_10_10_10_10_10_10);
      let byte4 = 0x08_08_08_08_08_08_08_08;
      assert_eq!(byte4.move_up_mask(1), 0x08_08_08_08_08_08_08);
      let byte5 = 0x04_04_04_04_04_04_04_04;
      assert_eq!(byte5.move_up_mask(1), 0x04_04_04_04_04_04_04);
      let byte6 = 0x02_02_02_02_02_02_02_02;
      assert_eq!(byte6.move_up_mask(1), 0x02_02_02_02_02_02_02);
      let byte7 = 0x01_01_01_01_01_01_01_01;
      assert_eq!(byte7.move_up_mask(1), 0x01_01_01_01_01_01_01);
    }

    #[test]
    fn ver_byte_move_by_eight() {
      let byte0 = 0x80_80_80_80_80_80_80_80;
      assert_eq!(byte0.move_up_mask(8), 0);
      let byte1 = 0x40_40_40_40_40_40_40_40;
      assert_eq!(byte1.move_up_mask(8), 0);
      let byte2 = 0x20_20_20_20_20_20_20_20;
      assert_eq!(byte2.move_up_mask(8), 0);
      let byte3 = 0x10_10_10_10_10_10_10_10;
      assert_eq!(byte3.move_up_mask(8), 0);
      let byte4 = 0x08_08_08_08_08_08_08_08;
      assert_eq!(byte4.move_up_mask(8), 0);
      let byte5 = 0x04_04_04_04_04_04_04_04;
      assert_eq!(byte5.move_up_mask(8), 0);
      let byte6 = 0x02_02_02_02_02_02_02_02;
      assert_eq!(byte6.move_up_mask(8), 0);
      let byte7 = 0x01_01_01_01_01_01_01_01;
      assert_eq!(byte7.move_up_mask(8), 0);
    }

    #[test]
    fn ver_byte_move_to_out_of_bounds() {
      let byte0 = 0x80_80_80_80_80_80_80_80;
      assert_eq!(byte0.move_up_mask(8), 0);
      let byte1 = 0x40_40_40_40_40_40_40;
      assert_eq!(byte1.move_up_mask(7), 0);
      let byte2 = 0x20_20_20_20_20_20;
      assert_eq!(byte2.move_up_mask(6), 0);
      let byte3 = 0x10_10_10_10_10;
      assert_eq!(byte3.move_up_mask(5), 0);
      let byte4 = 0x08_08_08_08;
      assert_eq!(byte4.move_up_mask(4), 0);
      let byte5 = 0x04_04_04;
      assert_eq!(byte5.move_up_mask(3), 0);
      let byte6 = 0x02_02;
      assert_eq!(byte6.move_up_mask(2), 0);
      let byte7 = 0x01;
      assert_eq!(byte7.move_up_mask(1), 0);
    }
  }

  mod down {
    use super::BoardMovement;

    #[test]
    fn bit_move_by_zero() {
      let bit0 = 0x80;
      assert_eq!(bit0.move_down_mask(0), bit0);
      let bit1 = 0x40;
      assert_eq!(bit1.move_down_mask(0), bit1);
      let bit2 = 0x20;
      assert_eq!(bit2.move_down_mask(0), bit2);
      let bit3 = 0x10;
      assert_eq!(bit3.move_down_mask(0), bit3);
      let bit4 = 0x08;
      assert_eq!(bit4.move_down_mask(0), bit4);
      let bit5 = 0x04;
      assert_eq!(bit5.move_down_mask(0), bit5);
      let bit6 = 0x02;
      assert_eq!(bit6.move_down_mask(0), bit6);
      let bit7 = 0x01;
      assert_eq!(bit7.move_down_mask(0), bit7);
    }

    #[test]
    fn bit_move_by_one() {
      let bit0 = 0x80;
      assert_eq!(bit0.move_down_mask(1), 0x80_00);
      let bit1 = 0x40;
      assert_eq!(bit1.move_down_mask(1), 0x40_00);
      let bit2 = 0x20;
      assert_eq!(bit2.move_down_mask(1), 0x20_00);
      let bit3 = 0x10;
      assert_eq!(bit3.move_down_mask(1), 0x10_00);
      let bit4 = 0x08;
      assert_eq!(bit4.move_down_mask(1), 0x08_00);
      let bit5 = 0x04;
      assert_eq!(bit5.move_down_mask(1), 0x04_00);
      let bit6 = 0x02;
      assert_eq!(bit6.move_down_mask(1), 0x02_00);
      let bit7 = 0x01;
      assert_eq!(bit7.move_down_mask(1), 0x01_00);
    }

    #[test]
    fn bit_move_by_eight() {
      let bit0 = 0x80;
      assert_eq!(bit0.move_down_mask(8), 0);
      let bit1 = 0x40;
      assert_eq!(bit1.move_down_mask(8), 0);
      let bit2 = 0x20;
      assert_eq!(bit2.move_down_mask(8), 0);
      let bit3 = 0x10;
      assert_eq!(bit3.move_down_mask(8), 0);
      let bit4 = 0x08;
      assert_eq!(bit4.move_down_mask(8), 0);
      let bit5 = 0x04;
      assert_eq!(bit5.move_down_mask(8), 0);
      let bit6 = 0x02;
      assert_eq!(bit6.move_down_mask(8), 0);
      let bit7 = 0x01;
      assert_eq!(bit7.move_down_mask(8), 0);
    }

    #[test]
    fn bit_move_to_out_of_bounds() {
      let bit0 = 0x80;
      assert_eq!(bit0.move_down_mask(8), 0);
      let bit1 = 0x40_00;
      assert_eq!(bit1.move_down_mask(7), 0);
      let bit2 = 0x20_00_00;
      assert_eq!(bit2.move_down_mask(6), 0);
      let bit3 = 0x10_00_00_00;
      assert_eq!(bit3.move_down_mask(5), 0);
      let bit4 = 0x08_00_00_00_00;
      assert_eq!(bit4.move_down_mask(4), 0);
      let bit5 = 0x04_00_00_00_00_00;
      assert_eq!(bit5.move_down_mask(3), 0);
      let bit6 = 0x02_00_00_00_00_00_00;
      assert_eq!(bit6.move_down_mask(2), 0);
      let bit7 = 0x01_00_00_00_00_00_00_00;
      assert_eq!(bit7.move_down_mask(1), 0);
    }

    #[test]
    fn ver_byte_move_by_zero() {
      let byte0 = 0x80_80_80_80_80_80_80_80;
      assert_eq!(byte0.move_down_mask(0), byte0);
      let byte1 = 0x40_40_40_40_40_40_40_40;
      assert_eq!(byte1.move_down_mask(0), byte1);
      let byte2 = 0x20_20_20_20_20_20_20_20;
      assert_eq!(byte2.move_down_mask(0), byte2);
      let byte3 = 0x10_10_10_10_10_10_10_10;
      assert_eq!(byte3.move_down_mask(0), byte3);
      let byte4 = 0x08_08_08_08_08_08_08_08;
      assert_eq!(byte4.move_down_mask(0), byte4);
      let byte5 = 0x04_04_04_04_04_04_04_04;
      assert_eq!(byte5.move_down_mask(0), byte5);
      let byte6 = 0x02_02_02_02_02_02_02_02;
      assert_eq!(byte6.move_down_mask(0), byte6);
      let byte7 = 0x01_01_01_01_01_01_01_01;
      assert_eq!(byte7.move_down_mask(0), byte7);
    }

    #[test]
    fn ver_byte_move_by_one() {
      let byte0 = 0x80_80_80_80_80_80_80_80;
      assert_eq!(byte0.move_down_mask(1), 0x80_80_80_80_80_80_80_00);
      let byte1 = 0x40_40_40_40_40_40_40_40;
      assert_eq!(byte1.move_down_mask(1), 0x40_40_40_40_40_40_40_00);
      let byte2 = 0x20_20_20_20_20_20_20_20;
      assert_eq!(byte2.move_down_mask(1), 0x20_20_20_20_20_20_20_00);
      let byte3 = 0x10_10_10_10_10_10_10_10;
      assert_eq!(byte3.move_down_mask(1), 0x10_10_10_10_10_10_10_00);
      let byte4 = 0x08_08_08_08_08_08_08_08;
      assert_eq!(byte4.move_down_mask(1), 0x08_08_08_08_08_08_08_00);
      let byte5 = 0x04_04_04_04_04_04_04_04;
      assert_eq!(byte5.move_down_mask(1), 0x04_04_04_04_04_04_04_00);
      let byte6 = 0x02_02_02_02_02_02_02_02;
      assert_eq!(byte6.move_down_mask(1), 0x02_02_02_02_02_02_02_00);
      let byte7 = 0x01_01_01_01_01_01_01_01;
      assert_eq!(byte7.move_down_mask(1), 0x01_01_01_01_01_01_01_00);
    }

    #[test]
    fn ver_byte_move_by_eight() {
      let byte0 = 0x80_80_80_80_80_80_80_80;
      assert_eq!(byte0.move_down_mask(8), 0);
      let byte1 = 0x40_40_40_40_40_40_40_40;
      assert_eq!(byte1.move_down_mask(8), 0);
      let byte2 = 0x20_20_20_20_20_20_20_20;
      assert_eq!(byte2.move_down_mask(8), 0);
      let byte3 = 0x10_10_10_10_10_10_10_10;
      assert_eq!(byte3.move_down_mask(8), 0);
      let byte4 = 0x08_08_08_08_08_08_08_08;
      assert_eq!(byte4.move_down_mask(8), 0);
      let byte5 = 0x04_04_04_04_04_04_04_04;
      assert_eq!(byte5.move_down_mask(8), 0);
      let byte6 = 0x02_02_02_02_02_02_02_02;
      assert_eq!(byte6.move_down_mask(8), 0);
      let byte7 = 0x01_01_01_01_01_01_01_01;
      assert_eq!(byte7.move_down_mask(8), 0);
    }

    #[test]
    fn ver_byte_move_to_out_of_bounds() {
      let byte0 = 0x80_80_80_80_80_80_80_80;
      assert_eq!(byte0.move_down_mask(8), 0);
      let byte1 = 0x40_40_40_40_40_40_40_00;
      assert_eq!(byte1.move_down_mask(7), 0);
      let byte2 = 0x20_20_20_20_20_20_00_00;
      assert_eq!(byte2.move_down_mask(6), 0);
      let byte3 = 0x10_10_10_10_10_00_00_00;
      assert_eq!(byte3.move_down_mask(5), 0);
      let byte4 = 0x08_08_08_08_00_00_00_00;
      assert_eq!(byte4.move_down_mask(4), 0);
      let byte5 = 0x04_04_04_00_00_00_00_00;
      assert_eq!(byte5.move_down_mask(3), 0);
      let byte6 = 0x02_02_00_00_00_00_00_00;
      assert_eq!(byte6.move_down_mask(2), 0);
      let byte7 = 0x01_00_00_00_00_00_00_00;
      assert_eq!(byte7.move_down_mask(1), 0);
    }
  }

  mod complex {
    use super::BoardMovement;

    #[test]
    fn snake() {
      let mut num = 0x80;
      num = num.move_right_mask(7);
      assert_eq!(num, 0x01);
      num = num.move_down_mask(1);
      assert_eq!(num, 0x01_00);
      num = num.move_left_mask(7);
      assert_eq!(num, 0x80_00);
      num = num.move_down_mask(1);
      assert_eq!(num, 0x80_00_00);
      num = num.move_right_mask(7);
      assert_eq!(num, 0x01_00_00);
      num = num.move_down_mask(1);
      assert_eq!(num, 0x01_00_00_00);
      num = num.move_left_mask(7);
      assert_eq!(num, 0x80_00_00_00);
      num = num.move_down_mask(1);
      assert_eq!(num, 0x80_00_00_00_00);
      num = num.move_right_mask(7);
      assert_eq!(num, 0x01_00_00_00_00);
      num = num.move_down_mask(1);
      assert_eq!(num, 0x01_00_00_00_00_00);
      num = num.move_left_mask(7);
      assert_eq!(num, 0x80_00_00_00_00_00);
      num = num.move_down_mask(1);
      assert_eq!(num, 0x80_00_00_00_00_00_00);
      num = num.move_right_mask(7);
      assert_eq!(num, 0x01_00_00_00_00_00_00);
      num = num.move_down_mask(1);
      assert_eq!(num, 0x01_00_00_00_00_00_00_00);
      num = num.move_left_mask(7);
      assert_eq!(num, 0x80_00_00_00_00_00_00_00);
      num = num.move_down_mask(1);
      assert_eq!(num, 0);
    }

    #[test]
    fn hor_shake() {
      let mut num = u64::MAX;
      num = num.move_right_mask(1);
      assert_eq!(num, 0x7F_7F_7F_7F_7F_7F_7F_7F);
      num = num.move_left_mask(2);
      assert_eq!(num, 0xFC_FC_FC_FC_FC_FC_FC_FC);
      num = num.move_right_mask(3);
      assert_eq!(num, 0x1F_1F_1F_1F_1F_1F_1F_1F);
      num = num.move_left_mask(4);
      assert_eq!(num, 0xF0_F0_F0_F0_F0_F0_F0_F0);
      num = num.move_right_mask(5);
      assert_eq!(num, 0x07_07_07_07_07_07_07_07);
      num = num.move_left_mask(6);
      assert_eq!(num, 0xC0_C0_C0_C0_C0_C0_C0_C0);
      num = num.move_right_mask(7);
      assert_eq!(num, 0x01_01_01_01_01_01_01_01);
      num = num.move_left_mask(8);
      assert_eq!(num, 0);
    }

    #[test]
    fn ver_shake() {
      let mut num = u64::MAX;
      num = num.move_up_mask(1);
      assert_eq!(num, 0x00_FF_FF_FF_FF_FF_FF_FF);
      num = num.move_down_mask(2);
      assert_eq!(num, 0xFF_FF_FF_FF_FF_FF_00_00);
      num = num.move_up_mask(3);
      assert_eq!(num, 0x00_00_00_FF_FF_FF_FF_FF);
      num = num.move_down_mask(4);
      assert_eq!(num, 0xFF_FF_FF_FF_00_00_00_00);
      num = num.move_up_mask(5);
      assert_eq!(num, 0x00_00_00_00_00_FF_FF_FF);
      num = num.move_down_mask(6);
      assert_eq!(num, 0xFF_FF_00_00_00_00_00_00);
      num = num.move_up_mask(7);
      assert_eq!(num, 0x00_00_00_00_00_00_00_FF);
      num = num.move_down_mask(8);
      assert_eq!(num, 0);
    }
  }
}
