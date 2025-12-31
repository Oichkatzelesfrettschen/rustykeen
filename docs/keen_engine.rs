// docs/keen_engine.rs
// Rust trait/struct stubs matching docs/uniffi.udl (namespace keen)

pub mod keen {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub enum Op {
        Add,
        Sub,
        Mul,
        Div,
        Eq,
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct Cell {
        pub row: u8,
        pub col: u8,
        pub value: u8, // 0 = empty
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct CageSpec {
        pub cells: Vec<Cell>,
        pub op: Op,
        pub target: u32,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct PuzzleSpec {
        pub size: u8, // 4..9
        pub cages: Vec<CageSpec>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct PuzzleState {
        pub cells: Vec<u8>, // flattened size*size, 0 empty
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct DifficultyMetrics {
        pub steps: u32,
        pub tier: u8, // 0 easy .. 3 extreme
    }

    // Trait matching UDL interface methods; UniFFI will bind to impl on a concrete type
    pub trait KeenEngine {
        fn generate_puzzle(&self, seed: u64, size: u8, difficulty: u8) -> PuzzleState;
        fn solve_puzzle(&self, spec: PuzzleSpec, stepwise: bool) -> Option<PuzzleState>;
        fn estimate_difficulty(&self, spec: PuzzleSpec) -> DifficultyMetrics;
        fn generate_batch(&self, seed: u64, size: u8, difficulty: u8, count: u32) -> Vec<PuzzleState>;
    }

    // Minimal stub implementation; replace todo!() with real logic
    pub struct KeenEngineImpl;

    impl KeenEngineImpl {
        pub fn new() -> Self { Self }
    }

    impl KeenEngine for KeenEngineImpl {
        fn generate_puzzle(&self, _seed: u64, size: u8, _difficulty: u8) -> PuzzleState {
            PuzzleState { cells: vec![0; (size as usize) * (size as usize)] }
        }
        fn solve_puzzle(&self, spec: PuzzleSpec, _stepwise: bool) -> Option<PuzzleState> {
            Some(PuzzleState { cells: vec![0; (spec.size as usize) * (spec.size as usize)] })
        }
        fn estimate_difficulty(&self, _spec: PuzzleSpec) -> DifficultyMetrics {
            DifficultyMetrics { steps: 0, tier: 0 }
        }
        fn generate_batch(&self, seed: u64, size: u8, difficulty: u8, count: u32) -> Vec<PuzzleState> {
            (0..count).map(|_| self.generate_puzzle(seed, size, difficulty)).collect()
        }
    }
}
