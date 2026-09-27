use weather_buoys::arrays::{denan, mean};

const EPSILON: f32 = 1.0e-6;

fn assert_f32_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= EPSILON,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn denan_returns_empty_for_empty_input() {
    let input: [f32; 0] = [];

    assert_eq!(denan(&input), Vec::<f32>::new());
}

#[test]
fn denan_returns_empty_when_all_values_are_nan() {
    let input = [f32::NAN, f32::NAN, f32::NAN];

    assert!(denan(&input).is_empty());
}

#[test]
fn denan_removes_nan_values() {
    let input = [1.0, f32::NAN, 2.5, f32::NAN, -3.0];

    assert_eq!(denan(&input), vec![1.0, 2.5, -3.0]);
}

#[test]
fn denan_preserves_the_order_of_non_nan_values() {
    let input = [
        10.0,
        f32::NAN,
        -4.5,
        f32::INFINITY,
        f32::NAN,
        f32::NEG_INFINITY,
        0.0,
    ];

    assert_eq!(
        denan(&input),
        vec![10.0, -4.5, f32::INFINITY, f32::NEG_INFINITY, 0.0]
    );
}

#[test]
fn denan_keeps_infinities() {
    let input = [f32::INFINITY, f32::NEG_INFINITY, f32::NAN];

    assert_eq!(denan(&input), vec![f32::INFINITY, f32::NEG_INFINITY]);
}

#[test]
fn denan_keeps_positive_and_negative_zero() {
    let input = [0.0_f32, -0.0_f32];

    let result = denan(&input);

    assert_eq!(result.len(), 2);
    assert!(!result[0].is_sign_negative());
    assert!(result[1].is_sign_negative());
}

#[test]
fn denan_does_not_modify_the_input() {
    let input = [1.0, f32::NAN, 2.0];

    let _ = denan(&input);

    assert_eq!(input[0], 1.0);
    assert!(input[1].is_nan());
    assert_eq!(input[2], 2.0);
}

#[test]
fn denan_returns_an_independent_vector() {
    let input = [1.0, 2.0];

    let mut result = denan(&input);
    result[0] = 99.0;

    assert_eq!(input, [1.0, 2.0]);
    assert_eq!(result, vec![99.0, 2.0]);
}

#[test]
fn mean_returns_none_for_empty_input() {
    let input: [f32; 0] = [];

    assert_eq!(mean(&input), None);
}

#[test]
fn mean_returns_the_single_value_for_singleton_input() {
    assert_eq!(mean(&[42.5]), Some(42.5));
}

#[test]
fn mean_calculates_the_arithmetic_mean() {
    let input = [1.0, 2.0, 3.0, 4.0, 5.0];

    assert_eq!(mean(&input), Some(3.0));
}

#[test]
fn mean_handles_negative_and_fractional_values() {
    let input = [-2.5, 1.5, 4.0];

    assert_f32_eq(mean(&input).unwrap(), 1.0);
}

#[test]
fn mean_handles_zero_sum() {
    let input = [-10.0, 10.0];

    assert_eq!(mean(&input), Some(0.0));
}

#[test]
fn mean_handles_infinity_when_the_result_is_defined() {
    assert_eq!(mean(&[f32::INFINITY, 1.0]), Some(f32::INFINITY));
    assert_eq!(mean(&[f32::NEG_INFINITY, -1.0]), Some(f32::NEG_INFINITY));
}

#[test]
fn mean_returns_nan_when_input_contains_nan() {
    let result = mean(&[1.0, f32::NAN, 3.0]);

    assert!(result.unwrap().is_nan());
}

#[test]
fn mean_returns_nan_for_opposite_infinities() {
    let result = mean(&[f32::INFINITY, f32::NEG_INFINITY]);

    assert!(result.unwrap().is_nan());
}

#[test]
fn mean_does_not_modify_the_input() {
    let input = [1.0, 2.0, 3.0];
    let original = input;

    let _ = mean(&input);

    assert_eq!(input, original);
}

#[test]
fn mean_uses_all_values_including_nan_filtering_is_not_implicit() {
    let input = [1.0, f32::NAN, 3.0];

    // `mean` is specified to average the supplied slice directly.
    // It must not silently behave like `mean(&denan(input))`.
    let result = mean(&input).unwrap();

    assert!(result.is_nan());
}
