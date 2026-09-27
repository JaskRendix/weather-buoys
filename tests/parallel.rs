use weather_buoys::parallel::{parallel_mean, tile_indices};

const EPSILON: f32 = 1.0e-5;

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= EPSILON,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn tile_indices_returns_none_when_worker_count_is_zero() {
    assert_eq!(tile_indices(10, 0, 0), None);
}

#[test]
fn tile_indices_returns_none_for_worker_id_out_of_bounds() {
    assert_eq!(tile_indices(10, 3, 3), None);
    assert_eq!(tile_indices(10, 3, usize::MAX), None);
}

#[test]
fn tile_indices_returns_the_entire_range_for_one_worker() {
    assert_eq!(tile_indices(10, 1, 0), Some(0..10));
}

#[test]
fn tile_indices_returns_empty_range_for_zero_dimensions() {
    assert_eq!(tile_indices(0, 3, 0), Some(0..0));
    assert_eq!(tile_indices(0, 3, 1), Some(0..0));
    assert_eq!(tile_indices(0, 3, 2), Some(0..0));
}

#[test]
fn tile_indices_divides_evenly() {
    assert_eq!(tile_indices(12, 3, 0), Some(0..4));
    assert_eq!(tile_indices(12, 3, 1), Some(4..8));
    assert_eq!(tile_indices(12, 3, 2), Some(8..12));
}

#[test]
fn tile_indices_distributes_remainder_to_first_workers() {
    // 10 elements divided among 3 workers:
    // worker 0 gets 4, workers 1 and 2 get 3.
    assert_eq!(tile_indices(10, 3, 0), Some(0..4));
    assert_eq!(tile_indices(10, 3, 1), Some(4..7));
    assert_eq!(tile_indices(10, 3, 2), Some(7..10));
}

#[test]
fn tile_indices_handles_more_workers_than_dimensions() {
    assert_eq!(tile_indices(2, 5, 0), Some(0..1));
    assert_eq!(tile_indices(2, 5, 1), Some(1..2));
    assert_eq!(tile_indices(2, 5, 2), Some(2..2));
    assert_eq!(tile_indices(2, 5, 3), Some(2..2));
    assert_eq!(tile_indices(2, 5, 4), Some(2..2));
}

#[test]
fn tile_indices_ranges_are_contiguous_and_cover_all_elements() {
    let dims = 17;
    let workers = 5;

    let ranges: Vec<_> = (0..workers)
        .map(|worker_id| tile_indices(dims, workers, worker_id).unwrap())
        .collect();

    assert_eq!(ranges[0], 0..4);
    assert_eq!(ranges[1], 4..8);
    assert_eq!(ranges[2], 8..11);
    assert_eq!(ranges[3], 11..14);
    assert_eq!(ranges[4], 14..17);

    for pair in ranges.windows(2) {
        assert_eq!(pair[0].end, pair[1].start);
    }

    assert_eq!(ranges.first().unwrap().start, 0);
    assert_eq!(ranges.last().unwrap().end, dims);
}

#[test]
fn tile_indices_each_element_belongs_to_exactly_one_range() {
    let dims = 23;
    let workers = 7;

    let ranges: Vec<_> = (0..workers)
        .map(|worker_id| tile_indices(dims, workers, worker_id).unwrap())
        .collect();

    for index in 0..dims {
        let containing_ranges = ranges.iter().filter(|range| range.contains(&index)).count();

        assert_eq!(
            containing_ranges, 1,
            "index {index} was not assigned exactly once"
        );
    }
}

#[test]
fn tile_indices_ranges_do_not_exceed_dimensions() {
    for dims in 0..30 {
        for workers in 1..10 {
            for worker_id in 0..workers {
                let range = tile_indices(dims, workers, worker_id).unwrap();

                assert!(range.start <= dims);
                assert!(range.end <= dims);
                assert!(range.start <= range.end);
            }
        }
    }
}

#[test]
fn parallel_mean_returns_none_for_empty_input() {
    assert_eq!(parallel_mean(&[]), None);
}

#[test]
fn parallel_mean_returns_none_when_all_values_are_nan() {
    let values = [f32::NAN, f32::NAN, f32::NAN];

    assert_eq!(parallel_mean(&values), None);
}

#[test]
fn parallel_mean_returns_the_single_value() {
    assert_eq!(parallel_mean(&[42.5]), Some(42.5));
}

#[test]
fn parallel_mean_ignores_nan_values() {
    let values = [1.0, f32::NAN, 3.0, f32::NAN, 5.0];

    assert_close(parallel_mean(&values).unwrap(), 3.0);
}

#[test]
fn parallel_mean_calculates_the_arithmetic_mean() {
    let values = [1.0, 2.0, 3.0, 4.0, 5.0];

    assert_close(parallel_mean(&values).unwrap(), 3.0);
}

#[test]
fn parallel_mean_handles_negative_and_fractional_values() {
    let values = [-2.5, 1.5, 4.0];

    assert_close(parallel_mean(&values).unwrap(), 1.0);
}

#[test]
fn parallel_mean_handles_zero_sum() {
    let values = [-10.0, 10.0];

    assert_eq!(parallel_mean(&values), Some(0.0));
}

#[test]
fn parallel_mean_handles_infinity_when_the_result_is_defined() {
    assert_eq!(parallel_mean(&[f32::INFINITY, 1.0]), Some(f32::INFINITY));

    assert_eq!(
        parallel_mean(&[f32::NEG_INFINITY, -1.0]),
        Some(f32::NEG_INFINITY)
    );
}

#[test]
fn parallel_mean_returns_nan_for_opposite_infinities() {
    let result = parallel_mean(&[f32::INFINITY, f32::NEG_INFINITY]);

    assert!(result.unwrap().is_nan());
}

#[test]
fn parallel_mean_ignores_nan_but_not_infinity() {
    let result = parallel_mean(&[f32::NAN, f32::INFINITY, 10.0]);

    assert_eq!(result, Some(f32::INFINITY));
}

#[test]
fn parallel_mean_does_not_modify_the_input() {
    let values = [1.0, f32::NAN, 3.0];
    let original = values;

    let _ = parallel_mean(&values);

    assert_eq!(values[0], original[0]);
    assert!(values[1].is_nan());
    assert_eq!(values[2], original[2]);
}

#[test]
fn parallel_mean_works_with_a_large_input() {
    let values: Vec<f32> = (1..=10_000).map(|value| value as f32).collect();

    let result = parallel_mean(&values).unwrap();

    assert_close(result, 5000.5);
}
