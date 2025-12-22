// Custom Assertions for Integration Tests

use reqwest::{Response, StatusCode};
use serde::de::DeserializeOwned;

/// Assert response has expected status code
pub async fn assert_status(response: Response, expected: StatusCode) -> Response {
    let actual = response.status();
    assert_eq!(
        actual, expected,
        "Expected status {}, got {}",
        expected, actual
    );
    response
}

/// Assert response is successful (2xx)
pub async fn assert_success(response: Response) -> Response {
    let status = response.status();
    assert!(
        status.is_success(),
        "Expected successful response, got {}",
        status
    );
    response
}

/// Assert response is client error (4xx)
pub async fn assert_client_error(response: Response) -> Response {
    let status = response.status();
    assert!(
        status.is_client_error(),
        "Expected client error response, got {}",
        status
    );
    response
}

/// Assert response is server error (5xx)
pub async fn assert_server_error(response: Response) -> Response {
    let status = response.status();
    assert!(
        status.is_server_error(),
        "Expected server error response, got {}",
        status
    );
    response
}

/// Assert response contains JSON and deserialize
pub async fn assert_json<T: DeserializeOwned>(response: Response) -> T {
    response
        .json()
        .await
        .expect("Failed to deserialize JSON response")
}

/// Assert response body contains string
pub async fn assert_contains(response: Response, needle: &str) -> Response {
    let body = response.text().await.expect("Failed to read response body");
    assert!(
        body.contains(needle),
        "Expected response to contain '{}', got: {}",
        needle,
        body
    );
    // Note: Response is consumed, return a placeholder
    panic!("Response consumed by assert_contains")
}

/// Assert two values are approximately equal (for f64)
pub fn assert_approx_eq(actual: f64, expected: f64, tolerance: f64) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= tolerance,
        "Expected {} to be approximately equal to {} (tolerance: {}), difference: {}",
        actual,
        expected,
        tolerance,
        diff
    );
}

/// Assert vector contains element
pub fn assert_contains_element<T: PartialEq + std::fmt::Debug>(vec: &[T], element: &T) {
    assert!(
        vec.contains(element),
        "Expected vector to contain {:?}, got {:?}",
        element,
        vec
    );
}

/// Assert vector length
pub fn assert_length<T>(vec: &[T], expected: usize) {
    assert_eq!(
        vec.len(),
        expected,
        "Expected vector length {}, got {}",
        expected,
        vec.len()
    );
}

/// Assert option is some
pub fn assert_some<T>(opt: &Option<T>) -> &T {
    opt.as_ref().expect("Expected Some, got None")
}

/// Assert option is none
pub fn assert_none<T: std::fmt::Debug>(opt: &Option<T>) {
    assert!(opt.is_none(), "Expected None, got Some({:?})", opt);
}

/// Assert result is ok
pub fn assert_ok<T, E: std::fmt::Debug>(result: &Result<T, E>) -> &T {
    result.as_ref().expect("Expected Ok, got Err")
}

/// Assert result is err
pub fn assert_err<T: std::fmt::Debug, E>(result: &Result<T, E>) {
    assert!(result.is_err(), "Expected Err, got Ok({:?})", result);
}

/// Assert string matches regex
pub fn assert_matches_regex(text: &str, pattern: &str) {
    let re = regex::Regex::new(pattern).expect("Invalid regex pattern");
    assert!(
        re.is_match(text),
        "Expected text to match pattern '{}', got: {}",
        pattern,
        text
    );
}

/// Assert duration is within range
pub fn assert_duration_within(
    actual: std::time::Duration,
    min: std::time::Duration,
    max: std::time::Duration,
) {
    assert!(
        actual >= min && actual <= max,
        "Expected duration between {:?} and {:?}, got {:?}",
        min,
        max,
        actual
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_approx_eq() {
        assert_approx_eq(1.0, 1.0, 0.01);
        assert_approx_eq(1.0, 1.005, 0.01);
    }

    #[test]
    #[should_panic]
    fn test_assert_approx_eq_fails() {
        assert_approx_eq(1.0, 1.1, 0.01);
    }

    #[test]
    fn test_assert_contains_element() {
        let vec = vec![1, 2, 3, 4, 5];
        assert_contains_element(&vec, &3);
    }

    #[test]
    fn test_assert_length() {
        let vec = vec![1, 2, 3];
        assert_length(&vec, 3);
    }

    #[test]
    fn test_assert_some() {
        let opt = Some(42);
        assert_eq!(*assert_some(&opt), 42);
    }

    #[test]
    fn test_assert_none() {
        let opt: Option<i32> = None;
        assert_none(&opt);
    }
}
