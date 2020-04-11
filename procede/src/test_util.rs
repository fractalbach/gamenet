//! Module containing utility macros and functions for use in tests.

use std::fs as fs;
use serde::Serialize;

const TEST_OUT_DIR: &str = "target/test_out/";

/// Macro for checking that two vec2's are approximately equal.
///
/// This macro is effectively equivalent to checking
/// `(a.x - b.x).abs() < eps && (a.y - b.y) < eps`
///
/// # Arguments
/// * `a` - First vector
/// * `b` - Second vector
/// * `eps` - Optional epsilon argument. This is the maximum allowable
///             difference between a and b. Defaults to 1e-6.
#[macro_export]
macro_rules! assert_vec2_near {
    ($a:expr, $b:expr) => {{
        let eps = 1.0e-6;
        let (a, b) = (&$a, &$b);
        assert_vec2_near!(a, b, eps);
    }};
    ($a:expr, $b:expr, $eps:expr) => {{
        let (a, b) = (&$a, &$b);
        let eps = $eps;
        assert!(
            (a.x - b.x).abs() < eps && (a.y - b.y) < eps,
            "assertion failed: `(left !== right)` \
             (left: `({:?}, {:?})`, right: `({:?}, {:?})`, \
             expect diff: `{:?}`, real diff: `({:?}, {:?})`)",
            a.x,
            a.y,
            b.x,
            b.y,
            eps,
            (a.x - b.x).abs(),
            (a.y - b.y).abs(),
        );
    }};
}

/// Macro for checking that two vec3's are approximately equal.
///
/// This macro is effectively equivalent to checking
/// `(a.x - b.x).abs() < eps && (a.y - b.y) < eps`
///
/// # Arguments
/// * `a` - First vector
/// * `b` - Second vector
/// * `eps` - Optional epsilon argument. This is the maximum allowable
///             difference between a and b. Defaults to 1e-6.
#[macro_export]
macro_rules! assert_vec3_near {
    ($a:expr, $b:expr) => {{
        let eps = 1.0e-6;
        let (a, b) = (&$a, &$b);
        assert_vec3_near!(a, b, eps);
    }};
    ($a:expr, $b:expr, $eps:expr) => {{
        let (a, b) = (&$a, &$b);
        let eps = $eps;
        assert!(
            (a.x - b.x).abs() < eps && (a.y - b.y) < eps && (a.z - a.z) < eps,
            "assertion failed: `(left !== right)` \
             (left: `({:?}, {:?}, {:?})`, right: `({:?}, {:?}, {:?})`, \
             expect diff: `{:?}`, real diff: `({:?}, {:?}, {:?})`)",
            a.x, a.y, a.z,
            b.x, b.y, a.z,
            eps,
            (a.x - b.x).abs(), (a.y - b.y).abs(), (a.z - b.z).abs(),
        );
    }};
}

/// Checks that a is less than b.
///
/// # Arguments
/// * a first value. Left side of comparison. Must be comparable to b
/// * b second value. Right side of comparison.
#[macro_export]
macro_rules! assert_lt {
    ($a:expr, $b:expr) => {{
        let (a, b) = (&$a, &$b);
        assert!(
            a < b,
            "assertion failed: `(left < right)` \
             (left: `{:?}`, right: `{:?}`",
            a,
            b,
        );
    }};
}

/// Checks that a is greater than b.
///
/// # Arguments
/// * a first value. Left side of comparison. Must be comparable to b
/// * b second value. Right side of comparison.
#[macro_export]
macro_rules! assert_gt {
    ($a:expr, $b:expr) => {{
        let (a, b) = (&$a, &$b);
        assert!(
            a > b,
            "assertion failed: `(left > right)` \
             (left: `{:?}`, right: `{:?}`",
            a,
            b,
        );
    }};
}

/// Checks that b is between a and c.
///
/// # Arguments
/// * a first value. Left side of comparison. Must be comparable to b
/// * b second value. Center of comparison.
/// * c second value. Right side of comparison.
#[macro_export]
macro_rules! assert_in_range {
    ($a:expr, $b:expr, $c:expr) => {{
        let (a, b, c) = (&$a, &$b, &$c);
        assert!(
            a <= b && b < c,
            "assertion failed: `(left <= mid < right)` \
             (left: `{:?}`, mid: `{:?}`, right: `{:?})`",
            a,
            b,
            c,
        );
    }};
}

#[macro_export]
macro_rules! assert_is {
    ($a:expr, $b:expr) => {{
        let (a, b) = (&$a, &$b);
        assert!(
            a as *const _ == b as *const _,
            "assertion failed: `left is not right` \
             (left: `{:?}`, right: `{:?}`)",
            a,
            b
        );
    }};
}

#[macro_export]
macro_rules! assert_is_not {
    ($a:expr, $b:expr) => {{
        let (a, b) = (&$a, &$b);
        assert!(
            a as *const _ != b as *const _,
            "assertion failed: `left is right` \
             (left: `{:?}`, right: `{:?}`)",
            a,
            b
        );
    }};
}

pub fn test_out(f_name: &str) -> String {
    fs::create_dir_all(TEST_OUT_DIR).ok();
    TEST_OUT_DIR.to_owned() + f_name
}

pub fn serialize_to<T: Serialize>(obj: &T, f_name: &str) {
    let s = serde_json::to_string_pretty(obj).expect(
        "Unable to serialize object."
    );
    fs::write(test_out(f_name), &s).expect(
        "Unable to write"
    );
}
