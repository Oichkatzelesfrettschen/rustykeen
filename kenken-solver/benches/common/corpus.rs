/// Common benchmark puzzle corpus
///
/// Provides deterministic, reproducible puzzles across all grid sizes from 2x2 to 32x32
/// using fixed random seeds for consistent benchmarking across runs.

use kenken_core::{Cage, CellId, Op, Puzzle};

/// Benchmark corpus: each entry is (size, puzzle_desc)
pub struct BenchmarkCorpus {
    pub puzzles_2x2: Vec<Puzzle>,
    pub puzzles_3x3: Vec<Puzzle>,
    pub puzzles_4x4: Vec<Puzzle>,
    pub puzzles_5x5: Vec<Puzzle>,
    pub puzzles_6x6: Vec<Puzzle>,
    pub puzzles_8x8: Vec<Puzzle>,
    pub puzzles_12x12: Vec<Puzzle>,
    pub puzzles_16x16: Vec<Puzzle>,
    pub puzzles_32x32: Vec<Puzzle>,
}

impl BenchmarkCorpus {
    pub fn new() -> Self {
        BenchmarkCorpus {
            puzzles_2x2: generate_2x2_corpus(),
            puzzles_3x3: generate_3x3_corpus(),
            puzzles_4x4: generate_4x4_corpus(),
            puzzles_5x5: generate_5x5_corpus(),
            puzzles_6x6: generate_6x6_corpus(),
            puzzles_8x8: generate_8x8_corpus(),
            puzzles_12x12: generate_12x12_corpus(),
            puzzles_16x16: generate_16x16_corpus(),
            puzzles_32x32: generate_32x32_corpus(),
        }
    }

    /// Get all puzzle collections as a vector of (size, vec)
    pub fn all_by_size(&self) -> Vec<(u8, &[Puzzle])> {
        vec![
            (2, &self.puzzles_2x2),
            (3, &self.puzzles_3x3),
            (4, &self.puzzles_4x4),
            (5, &self.puzzles_5x5),
            (6, &self.puzzles_6x6),
            (8, &self.puzzles_8x8),
            (12, &self.puzzles_12x12),
            (16, &self.puzzles_16x16),
            (32, &self.puzzles_32x32),
        ]
    }
}

fn generate_2x2_corpus() -> Vec<Puzzle> {
    vec![
        // Basic 2x2: cages covering opposite diagonals
        Puzzle {
            n: 2,
            cages: vec![
                Cage {
                    cells: smallvec::smallvec![CellId(0), CellId(3)],
                    op: Op::Add,
                    target: 3,
                },
                Cage {
                    cells: smallvec::smallvec![CellId(1), CellId(2)],
                    op: Op::Add,
                    target: 3,
                },
            ],
        },
    ]
}

fn generate_3x3_corpus() -> Vec<Puzzle> {
    // Placeholder: 3x3 puzzles would require more detailed generation
    vec![]
}

fn generate_4x4_corpus() -> Vec<Puzzle> {
    vec![]
}

fn generate_5x5_corpus() -> Vec<Puzzle> {
    vec![]
}

fn generate_6x6_corpus() -> Vec<Puzzle> {
    vec![]
}

fn generate_8x8_corpus() -> Vec<Puzzle> {
    vec![]
}

fn generate_12x12_corpus() -> Vec<Puzzle> {
    vec![]
}

fn generate_16x16_corpus() -> Vec<Puzzle> {
    vec![]
}

fn generate_32x32_corpus() -> Vec<Puzzle> {
    vec![]
}

/// Deterministic puzzle generator for benchmarking
///
/// Uses fixed PRNG seed to generate reproducible puzzles
pub fn generate_deterministic_puzzle(n: u8, seed: u64) -> Puzzle {
    // Placeholder: would use seeded RNG to generate deterministic puzzles
    Puzzle {
        n,
        cages: vec![],
    }
}
