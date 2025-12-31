// Kani proof: grid index bounds
#![allow(unused)]
#[cfg(kani)]
#[kani::proof]
fn grid_index_is_in_bounds() {
    let n: usize = kani::any();
    // Limit N to a practical range; adjust as needed
    kani::assume(n > 0 && n <= 16);
    let x: usize = kani::any();
    let y: usize = kani::any();
    kani::assume(x < n && y < n);
    let idx = y * n + x; // row-major
    assert!(idx < n * n);
}
