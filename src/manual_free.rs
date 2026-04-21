use std::mem::ManuallyDrop;

#[inline(always)]
pub fn free<T>(mut variable: ManuallyDrop<T>) {
  unsafe {
    ManuallyDrop::drop(&mut variable);
  }
}

#[inline(always)]
pub fn manual_free<T>(variable: T) -> ManuallyDrop<T> {
  ManuallyDrop::new(variable)
}
