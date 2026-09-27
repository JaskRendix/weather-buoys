//! Utility functions that operate on arrays.

/// Returns a new vector containing all non-NaN values from `x`.
pub fn denan(x: &[f32]) -> Vec<f32> {
    x.iter().copied().filter(|value| !value.is_nan()).collect()
}

/// Returns the arithmetic mean of `x`.
///
/// Returns `None` when the input is empty.
pub fn mean(x: &[f32]) -> Option<f32> {
    if x.is_empty() {
        None
    } else {
        Some(x.iter().sum::<f32>() / x.len() as f32)
    }
}
