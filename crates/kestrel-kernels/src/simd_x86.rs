//! AVX2 + FMA kernels for x86_64.
//!
//! Safety invariant for every function here: the caller (the dispatch layer
//! in `lib.rs`) has verified `avx2` and `fma` via `is_x86_feature_detected!`
//! before calling, and `a.len() == b.len()`. Loads are unaligned, so no
//! alignment assumption is made on the input slices.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Horizontal sum of the 8 lanes of an `__m256`.
///
/// # Safety
/// Requires AVX2 (verified by caller).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
unsafe fn hsum(v: __m256) -> f32 {
    let hi = _mm256_extractf128_ps::<1>(v);
    let lo = _mm256_castps256_ps128(v);
    let sum128 = _mm_add_ps(lo, hi);
    let shuf = _mm_movehl_ps(sum128, sum128);
    let sums = _mm_add_ps(sum128, shuf);
    let shuf2 = _mm_shuffle_ps::<0x01>(sums, sums);
    let result = _mm_add_ss(sums, shuf2);
    _mm_cvtss_f32(result)
}

/// AVX2 inner product. See module docs for the safety invariant.
///
/// # Safety
/// Requires AVX2+FMA and `a.len() == b.len()` (verified by caller).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
pub unsafe fn dot_avx2(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let n = a.len();
    let mut acc0 = _mm256_setzero_ps();
    let mut acc1 = _mm256_setzero_ps();
    let mut acc2 = _mm256_setzero_ps();
    let mut acc3 = _mm256_setzero_ps();
    let mut i = 0usize;
    // 32-wide main loop: four independent accumulators hide FMA latency.
    while i + 32 <= n {
        let a0 = _mm256_loadu_ps(a.as_ptr().add(i));
        let b0 = _mm256_loadu_ps(b.as_ptr().add(i));
        acc0 = _mm256_fmadd_ps(a0, b0, acc0);
        let a1 = _mm256_loadu_ps(a.as_ptr().add(i + 8));
        let b1 = _mm256_loadu_ps(b.as_ptr().add(i + 8));
        acc1 = _mm256_fmadd_ps(a1, b1, acc1);
        let a2 = _mm256_loadu_ps(a.as_ptr().add(i + 16));
        let b2 = _mm256_loadu_ps(b.as_ptr().add(i + 16));
        acc2 = _mm256_fmadd_ps(a2, b2, acc2);
        let a3 = _mm256_loadu_ps(a.as_ptr().add(i + 24));
        let b3 = _mm256_loadu_ps(b.as_ptr().add(i + 24));
        acc3 = _mm256_fmadd_ps(a3, b3, acc3);
        i += 32;
    }
    while i + 8 <= n {
        let av = _mm256_loadu_ps(a.as_ptr().add(i));
        let bv = _mm256_loadu_ps(b.as_ptr().add(i));
        acc0 = _mm256_fmadd_ps(av, bv, acc0);
        i += 8;
    }
    let acc = _mm256_add_ps(_mm256_add_ps(acc0, acc1), _mm256_add_ps(acc2, acc3));
    let mut total = hsum(acc);
    while i < n {
        total += a[i] * b[i];
        i += 1;
    }
    total
}

/// AVX2 squared L2 distance. See module docs for the safety invariant.
///
/// # Safety
/// Requires AVX2+FMA and `a.len() == b.len()` (verified by caller).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
pub unsafe fn l2_sq_avx2(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let n = a.len();
    let mut acc0 = _mm256_setzero_ps();
    let mut acc1 = _mm256_setzero_ps();
    let mut acc2 = _mm256_setzero_ps();
    let mut acc3 = _mm256_setzero_ps();
    let mut i = 0usize;
    while i + 32 <= n {
        let d0 = _mm256_sub_ps(
            _mm256_loadu_ps(a.as_ptr().add(i)),
            _mm256_loadu_ps(b.as_ptr().add(i)),
        );
        acc0 = _mm256_fmadd_ps(d0, d0, acc0);
        let d1 = _mm256_sub_ps(
            _mm256_loadu_ps(a.as_ptr().add(i + 8)),
            _mm256_loadu_ps(b.as_ptr().add(i + 8)),
        );
        acc1 = _mm256_fmadd_ps(d1, d1, acc1);
        let d2 = _mm256_sub_ps(
            _mm256_loadu_ps(a.as_ptr().add(i + 16)),
            _mm256_loadu_ps(b.as_ptr().add(i + 16)),
        );
        acc2 = _mm256_fmadd_ps(d2, d2, acc2);
        let d3 = _mm256_sub_ps(
            _mm256_loadu_ps(a.as_ptr().add(i + 24)),
            _mm256_loadu_ps(b.as_ptr().add(i + 24)),
        );
        acc3 = _mm256_fmadd_ps(d3, d3, acc3);
        i += 32;
    }
    while i + 8 <= n {
        let d = _mm256_sub_ps(
            _mm256_loadu_ps(a.as_ptr().add(i)),
            _mm256_loadu_ps(b.as_ptr().add(i)),
        );
        acc0 = _mm256_fmadd_ps(d, d, acc0);
        i += 8;
    }
    let acc = _mm256_add_ps(_mm256_add_ps(acc0, acc1), _mm256_add_ps(acc2, acc3));
    let mut total = hsum(acc);
    while i < n {
        let d = a[i] - b[i];
        total += d * d;
        i += 1;
    }
    total
}
