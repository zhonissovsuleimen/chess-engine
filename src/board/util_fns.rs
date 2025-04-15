pub fn if_bool(condition: bool, if_true: u64, if_false: u64) -> u64 {
  let condition_mask = mask_from_bool(condition);
  condition_mask & if_true | !condition_mask & if_false
}

pub fn if_mask(condition_mask: u64, if_true: u64, if_false: u64) -> u64 {
  condition_mask & if_true | !condition_mask & if_false
}

pub fn mask_from_bool(condition: bool) -> u64 {
  (condition as u64).wrapping_neg()
}