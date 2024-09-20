/// Utility function to check if `index` is in bounds for an array `[T; N]`.
///
/// Does nothing on success.
///
/// # Panics
///
/// If `index` is out of bounds.
#[inline]
pub(crate) const fn check_index<const LEN: usize>(index: usize) {
    let max_array = [(); LEN];
    let _ = &max_array[index];
}

/// Utility function to check if `[start..end]` is in bounds for an array `[T; N]`.
///
/// Does nothing on success.
///
/// # Panics
///
/// If `[start..end]` is out of bounds.
#[inline]
pub(crate) fn check_slice<const LEN: usize>(start: usize, end: usize) {
    let max_array = [(); LEN];
    let _ = &max_array[start..end];
}
