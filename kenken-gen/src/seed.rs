use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

pub fn rng_from_u64(seed: u64) -> ChaCha20Rng {
    // Deterministic across platforms and Rust versions for the same algorithm.
    // Uses the canonical SeedableRng mapping from `u64` to the full seed.
    ChaCha20Rng::seed_from_u64(seed)
}
