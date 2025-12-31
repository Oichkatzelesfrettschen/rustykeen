# Lazy DLX SolverContext (2025-12-31T22:07:20Z)

## Concept
- Keep DLX matrix pure (Latin). Use a lazy callback to verify arithmetic cages only when they become full during search.

## Data
- Bitmasks: cage_full[k], cage_cells[k] as masks over grid indices.
- Grid state: flattened cells; row/col conflict masks.

## API (Rust sketch)
```rust
pub struct SolverContext<'a> {
  pub n: u8,
  pub cages: &'a [CageSpec],
  pub cage_masks: Vec<bitvec::vec::BitVec>,
  pub cage_full: bitvec::vec::BitVec,
}

impl<'a> SolverContext<'a> {
  pub fn on_step(&mut self, grid: &[u8]) -> bool {
    // Check only newly-full cages
    for (k, mask) in self.cage_masks.iter().enumerate() {
      let full = mask.iter().all(|b| grid[b.index()] != 0);
      if full && !self.cage_full[k] {
        self.cage_full.set(k, true);
        if !verify_cage(&self.cages[k], grid) { return false; }
      }
    }
    true
  }
}
```

## Integration
- Register `on_step` with DLX search loop; backtrack when false.
