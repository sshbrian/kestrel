//! Distance kernels for Kestrel: the hot loops under every vector search.
//!
//! Design rules (see `docs/kernels.md` in the repository):
//!
//! - A plain scalar Rust reference exists for every kernel ([`scalar`]).
//! - Optimized paths (AVX2+FMA on x86_64, NEON on aarch64) sit behind safe
//!   wrappers with runtime CPU dispatch. All `unsafe` in the workspace lives
//!   in this crate.
//! - Every optimized path is differential-tested against the scalar
//!   reference: exact equality for integer kernels, a tight relative
//!   tolerance for `f32` kernels (SIMD changes the summation order, so
//!   bit-for-bit equality is not a property `f32` reductions can promise).
//! - A kernel that does not beat its scalar reference in `cargo bench`
//!   gets deleted.

pub mod scalar;

#[cfg(target_arch = "aarch64")]
mod simd_arm;
#[cfg(target_arch = "x86_64")]
mod simd_x86;

/// Squared L2 distance between two vectors, using the fastest kernel
/// available on this CPU.
///
/// # Panics
/// Panics if `a` and `b` have different lengths.
pub fn l2_sq(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "dimension mismatch");
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: AVX2+FMA verified just above; lengths verified by assert.
            return unsafe { simd_x86::l2_sq_avx2(a, b) };
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: NEON is baseline on supported aarch64 targets; lengths
        // verified by assert.
        return unsafe { simd_arm::l2_sq_neon(a, b) };
    }
    #[allow(unreachable_code)]
    scalar::l2_sq(a, b)
}

/// Inner product of two vectors, using the fastest kernel available.
///
/// # Panics
/// Panics if `a` and `b` have different lengths.
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "dimension mismatch");
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: AVX2+FMA verified just above; lengths verified by assert.
            return unsafe { simd_x86::dot_avx2(a, b) };
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: NEON is baseline on supported aarch64 targets; lengths
        // verified by assert.
        return unsafe { simd_arm::dot_neon(a, b) };
    }
    #[allow(unreachable_code)]
    scalar::dot(a, b)
}

/// Cosine similarity in `[-1, 1]`, built on the dispatched [`dot`].
/// Returns `0.0` if either vector is zero.
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
/// `u64::count_ones` compiles to a single POPCNT on supported targets,
/// so the reference implementation *is* the fast path until profiling
/// says otherwise.
///
/// # Panics
/// Panics if `a` and `b` have different lengths.
pub fn hamming(a: &[u64], b: &[u64]) -> u32 {
    scalar::hamming(a, b)
}

/// Human-readable name of the kernel path this CPU will take.
/// Used by demos and benchmark reports so numbers are never ambiguous
/// about which implementation produced them.
pub fn active_backend() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            return "avx2+fma";
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        return "neon";
    }
    #[allow(unreachable_code)]
    "scalar"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic xorshift64 PRNG: tests must not depend on an RNG crate
    /// or on wall-clock seeds.
    struct XorShift(u64);

    impl XorShift {
        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn next_f32(&mut self) -> f32 {
            // Uniform in [-1, 1).
            let v = (self.next_u64() >> 40) as f32 / (1u64 << 23) as f32;
            v - 1.0
        }
        fn vec(&mut self, n: usize) -> Vec<f32> {
            (0..n).map(|_| self.next_f32()).collect()
        }
    }

    fn assert_close(got: f32, want: f32, ctx: &str) {
        let tol = 1e-4 + 1e-4 * want.abs();
        assert!(
            (got - want).abs() <= tol,
            "{ctx}: got {got}, want {want} (tol {tol})"
        );
    }

    const DIMS: &[usize] = &[
        0, 1, 3, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 127, 128, 129, 384, 767, 768, 1000, 1536,
    ];

    #[test]
    fn dot_matches_scalar() {
        let mut rng = XorShift(0x9E37_79B9_7F4A_7C15);
        for &dim in DIMS {
            let a = rng.vec(dim);
            let b = rng.vec(dim);
            assert_close(dot(&a, &b), scalar::dot(&a, &b), &format!("dot dim={dim}"));
        }
    }

    #[test]
    fn l2_sq_matches_scalar() {
        let mut rng = XorShift(0x1234_5678_9ABC_DEF1);
        for &dim in DIMS {
            let a = rng.vec(dim);
            let b = rng.vec(dim);
            assert_close(
                l2_sq(&a, &b),
                scalar::l2_sq(&a, &b),
                &format!("l2_sq dim={dim}"),
            );
        }
    }

    #[test]
    fn cosine_matches_scalar() {
        let mut rng = XorShift(0x0BAD_5EED_0BAD_5EED);
        for &dim in DIMS {
            let a = rng.vec(dim);
            let b = rng.vec(dim);
            assert_close(
                cosine(&a, &b),
                scalar::cosine(&a, &b),
                &format!("cosine dim={dim}"),
            );
        }
    }

    #[test]
    fn cosine_zero_vector_is_zero() {
        let a = vec![0.0f32; 16];
        let b = vec![1.0f32; 16];
        assert_eq!(cosine(&a, &b), 0.0);
        assert_eq!(scalar::cosine(&a, &b), 0.0);
    }

    #[test]
    fn known_values() {
        let a = [1.0f32, 2.0, 3.0];
        let b = [4.0f32, 5.0, 6.0];
        assert_close(dot(&a, &b), 32.0, "dot known");
        assert_close(l2_sq(&a, &b), 27.0, "l2_sq known");
        assert_close(cosine(&a, &a), 1.0, "cosine self");
    }

    #[test]
    fn hamming_exact() {
        let mut rng = XorShift(0xFEED_FACE_1234_5678);
        for &words in &[0usize, 1, 2, 7, 16, 96] {
            let a: Vec<u64> = (0..words).map(|_| rng.next_u64()).collect();
            let b: Vec<u64> = (0..words).map(|_| rng.next_u64()).collect();
            assert_eq!(hamming(&a, &b), scalar::hamming(&a, &b));
            assert_eq!(hamming(&a, &a), 0);
        }
        assert_eq!(hamming(&[0b1010], &[0b0101]), 4);
    }

    #[test]
    #[should_panic(expected = "dimension mismatch")]
    fn dot_panics_on_mismatch() {
        dot(&[1.0, 2.0], &[1.0]);
    }
}
