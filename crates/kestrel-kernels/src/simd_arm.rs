//! NEON kernels for aarch64.
//!
//! NEON is part of the baseline for the `aarch64-unknown-linux-gnu` and
//! Apple-silicon targets Kestrel supports, so no runtime detection is needed
//! beyond the architecture itself. Safety invariant: `a.len() == b.len()`
//! (verified by the dispatch layer in `lib.rs`). Loads are unaligned.

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// NEON inner product. See module docs for the safety invariant.
///
/// # Safety
/// Requires NEON (baseline on supported aarch64 targets) and
/// `a.len() == b.len()` (verified by caller).
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn dot_neon(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let n = a.len();
    let mut acc0 = vdupq_n_f32(0.0);
    let mut acc1 = vdupq_n_f32(0.0);
    let mut acc2 = vdupq_n_f32(0.0);
    let mut acc3 = vdupq_n_f32(0.0);
    let mut i = 0usize;
    // 16-wide main loop: four independent accumulators hide FMLA latency.
    while i + 16 <= n {
        acc0 = vfmaq_f32(
            acc0,
            vld1q_f32(a.as_ptr().add(i)),
            vld1q_f32(b.as_ptr().add(i)),
        );
        acc1 = vfmaq_f32(
            acc1,
            vld1q_f32(a.as_ptr().add(i + 4)),
            vld1q_f32(b.as_ptr().add(i + 4)),
        );
        acc2 = vfmaq_f32(
            acc2,
            vld1q_f32(a.as_ptr().add(i + 8)),
            vld1q_f32(b.as_ptr().add(i + 8)),
        );
        acc3 = vfmaq_f32(
            acc3,
            vld1q_f32(a.as_ptr().add(i + 12)),
            vld1q_f32(b.as_ptr().add(i + 12)),
        );
        i += 16;
    }
    while i + 4 <= n {
        acc0 = vfmaq_f32(
            acc0,
            vld1q_f32(a.as_ptr().add(i)),
            vld1q_f32(b.as_ptr().add(i)),
        );
        i += 4;
    }
    let acc = vaddq_f32(vaddq_f32(acc0, acc1), vaddq_f32(acc2, acc3));
    let mut total = vaddvq_f32(acc);
    while i < n {
        total += a[i] * b[i];
        i += 1;
    }
    total
}

/// NEON squared L2 distance. See module docs for the safety invariant.
///
/// # Safety
/// Requires NEON (baseline on supported aarch64 targets) and
/// `a.len() == b.len()` (verified by caller).
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub unsafe fn l2_sq_neon(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let n = a.len();
    let mut acc0 = vdupq_n_f32(0.0);
    let mut acc1 = vdupq_n_f32(0.0);
    let mut acc2 = vdupq_n_f32(0.0);
    let mut acc3 = vdupq_n_f32(0.0);
    let mut i = 0usize;
    while i + 16 <= n {
        let d0 = vsubq_f32(vld1q_f32(a.as_ptr().add(i)), vld1q_f32(b.as_ptr().add(i)));
        acc0 = vfmaq_f32(acc0, d0, d0);
        let d1 = vsubq_f32(
            vld1q_f32(a.as_ptr().add(i + 4)),
            vld1q_f32(b.as_ptr().add(i + 4)),
        );
        acc1 = vfmaq_f32(acc1, d1, d1);
        let d2 = vsubq_f32(
            vld1q_f32(a.as_ptr().add(i + 8)),
            vld1q_f32(b.as_ptr().add(i + 8)),
        );
        acc2 = vfmaq_f32(acc2, d2, d2);
        let d3 = vsubq_f32(
            vld1q_f32(a.as_ptr().add(i + 12)),
            vld1q_f32(b.as_ptr().add(i + 12)),
        );
        acc3 = vfmaq_f32(acc3, d3, d3);
        i += 16;
    }
    while i + 4 <= n {
        let d = vsubq_f32(vld1q_f32(a.as_ptr().add(i)), vld1q_f32(b.as_ptr().add(i)));
        acc0 = vfmaq_f32(acc0, d, d);
        i += 4;
    }
    let acc = vaddq_f32(vaddq_f32(acc0, acc1), vaddq_f32(acc2, acc3));
    let mut total = vaddvq_f32(acc);
    while i < n {
        let d = a[i] - b[i];
        total += d * d;
        i += 1;
    }
    total
}
