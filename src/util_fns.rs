pub fn branchless_if(condition: bool, if_true: u64, if_false: u64) -> u64 {
  let condition_mask = mask_from_bool(condition);
  condition_mask & if_true | !condition_mask & if_false
}

pub fn mask_from_bool(condition: bool) -> u64 {
  !((condition as u64).wrapping_sub(1))
}