//! Module containing utility macros and functions for use in tests.


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
