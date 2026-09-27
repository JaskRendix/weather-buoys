use rayon::prelude::*;

/// Returns the half-open range assigned to a worker.
///
/// The returned range uses Rust's zero-based indexing:
///
/// ```text
/// start..end
/// ```
///
/// The first workers receive the smaller tiles, and any remainder
/// is distributed one element at a time.
pub fn tile_indices(
    dims: usize,
    num_workers: usize,
    worker_id: usize,
) -> Option<std::ops::Range<usize>> {
    if num_workers == 0 || worker_id >= num_workers {
        return None;
    }

    let base_size = dims / num_workers;
    let remainder = dims % num_workers;

    let start = worker_id * base_size + worker_id.min(remainder);
    let size = base_size + usize::from(worker_id < remainder);
    let end = start + size;

    Some(start..end)
}

/// Calculates the mean of non-NaN values in parallel.
pub fn parallel_mean(values: &[f32]) -> Option<f32> {
    let (sum, count) = values
        .par_iter()
        .filter(|value| !value.is_nan())
        .map(|value| (*value, 1usize))
        .reduce(
            || (0.0_f32, 0usize),
            |(sum_a, count_a), (sum_b, count_b)| (sum_a + sum_b, count_a + count_b),
        );

    if count == 0 {
        None
    } else {
        Some(sum / count as f32)
    }
}
