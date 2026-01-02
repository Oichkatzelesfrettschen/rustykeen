# SGT Puzzle Description Format

Last updated: 2026-01-01

## Overview

The sgt-desc format is Simon Tatham's compressed notation for puzzle layouts. It encodes:
1. **Block structure**: Which cells belong to the same cage (via edge boundaries)
2. **Clues**: Cage operations and target values

## Block Structure Encoding

### Edge Model

A KenKen puzzle grid can be represented as a graph where:
- **Nodes**: n×n cells
- **Edges**: Boundaries between adjacent cells (horizontal and vertical)

For an n×n grid:
- **Horizontal edges**: n rows × (n-1) edges per row = n(n-1) edges
- **Vertical edges**: (n-1) rows × n edges per row = n(n-1) edges
- **Total edges**: 2n(n-1)
- **Parse positions**: 2n(n-1) + 1 (the +1 is a sentinel for termination)

### Encoding Characters

The block structure is encoded as a sequence of characters representing edge states:

| Character | Meaning | Count |
|-----------|---------|-------|
| `_` (underscore) | Edge boundary (cells in different cages) | 1 |
| `a` | No-edge (1 cell in same cage) | 1 |
| `b` | No-edge (2 cells in same cage) | 2 |
| ...continuing... | ... | ... |
| `z` | No-edge (25 cells in same cage) | 25 |

### Run-Length Compression

To reduce size, identical consecutive characters are compressed:
- Single occurrence: output character only (e.g., `a` = one no-edge)
- Two occurrences: output character twice (e.g., `aa` = two no-edges)
- Three or more: output character + count (e.g., `a5` = five no-edges)

### Examples

**2×2 grid** (4 positions = 2×2×1 + 1 = 5):
- `_5` = underscore × 5 = all boundaries = each cell is a separate cage

**3×3 grid** (12 positions = 2×3×2 + 1 = 13):
- `_13` = underscore × 13 = all boundaries = singleton cages
- `aab_,c_d_,c_d_,e_f_,e6f_` = mixed cage layout

**5×5 grid** (40 positions = 2×5×4 + 1 = 41):
- `_41` = underscore × 41 = all boundaries = singleton cages

**6×6 grid** (60 positions = 2×6×5 + 1 = 61):
- `_61` = underscore × 61 = all boundaries = singleton cages

## Clue Format

After the block structure (delimited by comma), clues follow in order of cage minimum cell ID:

```
<operation><target>
```

### Operations

| Character | Operation |
|-----------|-----------|
| `a` | Addition (Sum) |
| `m` | Multiplication (Product) |
| `s` | Subtraction (only for 2-cell cages) |
| `d` | Division (only for 2-cell cages) |

### Target Values

- **Add/Mul**: Positive integers (e.g., `a6`, `m24`)
- **Sub**: Positive integers representing |a-b| (e.g., `s3`)
- **Div**: Positive integers representing max/min (e.g., `d2`)
- **Singleton cages**: Use `a` with the cell value (e.g., `a1`, `a5`)

### Examples

```
_41,a1a2a3a4a5a3a4a5a1a2a5a1a2a3a4a2a3a4a5a1a4a5a1a2a3
     ^                                                      ^
     └─ 25 addition clues for 25 singleton cages ──────────┘

__b,a3a3
 ^   ^^^^
 └─ Two non-edges with:
    - First cage: horizontal pair with target=3
    - Second cage: horizontal pair with target=3
```

## Grid Size Limits

- **Minimum**: n=1 (1×1 = trivial)
- **Maximum**: n=16 (256 cells; alphabet 'a'-'z' + extension)

Sizes n>26 require extended alphabet encoding beyond 'z'.

## Parsing Algorithm

1. **Parse block structure**: Read characters until comma
   - Characters: `_`, `a`-`z`
   - Optional digits after each character: repetition count
   - Validate total positions = 2n(n-1)+1

2. **Use Union-Find (DSU)**: Track which cells belong to same cage
   - Each edge character represents whether adjacent cells are connected
   - Horizontal edges processed first (row-major)
   - Vertical edges processed second (column-major transposed)

3. **Parse clues**: Read operation+target pairs
   - One clue per cage (in order of minimum cell ID)
   - Validate clue count matches cage count

## Validation Constraints

- Block structure must exactly consume 2n(n-1)+1 positions
- Clue count must match cage count
- Sub/Div operations only valid for 2-cell cages
- Target values must be valid integers

## Implementation Details

See: `kenken-core/src/format/sgt_desc.rs`
- `parse_keen_desc()` - Main parser
- `encode_keen_desc()` - Encoder (for testing)
- `parse_block_structure()` - Edge parsing logic
- `parse_clue()` - Clue parsing logic
- `compress_runs()` - Run-length compression

## References

- Upstream: https://www.chiark.greenend.org.uk/~sgtatham/puzzles/
- Puzzle documentation: https://www.chiark.greenend.org.uk/~sgtatham/puzzles/keen.html
