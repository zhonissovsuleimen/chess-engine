pub(super) const NONE: u64 = 0;
pub(super) const EMPTY_IS_ENEMY: u64 = 1;
pub(super) const ALLY_IS_ENEMY: u64 = 2;

pub(super) fn empty_is_enemy(value: u64) -> bool {
  value & EMPTY_IS_ENEMY > 0
}

pub(super) fn ally_is_enemy(value: u64) -> bool {
  value & ALLY_IS_ENEMY > 0
}
