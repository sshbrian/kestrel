//! Scalar reference implementations.
//!
//! These are the correctness ground truth for every optimized path in this
//! crate. They are intentionally plain: no intrinsics, no unrolling tricks,
//! nothing a reader has to trust. Optimized kernels are differential-tested
//! against these functions (exact equality for integer kernels, a tight
//! relative tolerance for `f32` kernels, whose SIMD reduction order differs).

/// Squared L2 distance: `sum((a[i] - b[i])^2)`.
///
/// # Panics
/// Panics if `a` and `b` have different lengths.
pub fn l2_sq(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "dimension mismatch");
    let mut acc = 0.0f32;
    for i in 0..a.len() {
        let d = a[i] - b[i];
        acc += d * d;
    }
    acc
}

/// Inner product: `sum(a[i] * b[i])`.
///
/// # Panics
/// Panics if `a` and `b` have different lengths.
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "dimension mismatch");
    let mut acc = 0.0f32;
    for i in 0..a.len() {
        acc += a[i] * b[i];
    }
    acc
}

/// Cosine similarity in `[-1, 1]`. Returns `0.0` if either vector is zero.
///
/// # Panics
/// Panics if `a` and `b` have different lengths.
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "dimension mismatch");
    let denom = dot(a, a).sqrt() * dot(b, b).sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot(a, b) / denom
    }
}

/// Hamming distance between two bit vectors packed into `u64` words.
///
/// # Panics
/// Panics if `a` and `b` have different lengths.
pub fn hamming(a: &[u64], b: &[u64]) -> u32 {
    assert_eq!(a.len(), b.len(), "dimension mismatch");
    let mut acc = 0u32;
    for i in 0..a.len() {
        acc += (a[i] ^ b[i]).count_ones();
    }
    acc
}
