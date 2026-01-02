//! Minimal Dancing Links (DLX) exact cover solver
//!
//! This is a cleanroom implementation of Knuth's Algorithm X using Dancing Links.
//! Internalized from dlx-rs to reduce external dependencies.
//!
//! References:
//! - Knuth, "Dancing Links" (2000): https://arxiv.org/pdf/cs/0011047.pdf
//! - Algorithm X for exact cover problems

/// A Dancing Links exact cover solver
///
/// Solves exact cover problems where you need to select a subset of options
/// such that each constraint is satisfied exactly once.
pub struct Solver<T> {
    /// Number of constraints (columns in the matrix)
    n_constraints: usize,
    /// Options with their associated data
    options: Vec<(T, Vec<usize>)>,
    /// Current state for iteration
    state: Option<SearchState>,
}

struct SearchState {
    /// Stack of (option_index, start_idx_for_next_level)
    stack: Vec<(usize, usize)>,
    /// Which constraints are currently covered
    covered: Vec<bool>,
    /// Have we finished?
    done: bool,
}

impl<T: Clone> Solver<T> {
    /// Create a new solver with the given number of constraints
    pub fn new(n_constraints: usize) -> Self {
        Self {
            n_constraints,
            options: Vec::new(),
            state: None,
        }
    }

    /// Add an option (row) that covers the given constraints (columns)
    pub fn add_option(&mut self, data: T, constraints: &[usize]) {
        self.options.push((data.clone(), constraints.to_vec()));
    }

    /// Find the next solution
    ///
    /// Returns Some(Vec<T>) with the selected options, or None if no more solutions exist.
    pub fn next(&mut self) -> Option<Vec<T>> {
        // Initialize state on first call
        if self.state.is_none() {
            self.state = Some(SearchState {
                stack: Vec::new(),
                covered: vec![false; self.n_constraints + 1],
                done: false,
            });
        }

        // Take state out to avoid borrow issues
        let mut state = self.state.take().unwrap();
        
        if state.done {
            self.state = Some(state);
            return None;
        }

        // Resume search from current state
        loop {
            // Check if all constraints are covered
            if (1..=self.n_constraints).all(|c| state.covered[c]) {
                // Found a solution - build result
                let solution: Vec<T> = state
                    .stack
                    .iter()
                    .map(|(opt_idx, _)| self.options[*opt_idx].0.clone())
                    .collect();

                // Backtrack one level to find next solution
                if !self.backtrack_one(&mut state) {
                    state.done = true;
                }

                self.state = Some(state);
                return Some(solution);
            }

            // Try to extend current solution
            let start_idx = state.stack.last().map(|(_, next)| *next).unwrap_or(0);
            
            if !self.try_extend(&mut state, start_idx) {
                // No more options at this level - backtrack
                if !self.backtrack_one(&mut state) {
                    state.done = true;
                    self.state = Some(state);
                    return None;
                }
            }
        }
    }

    fn try_extend(&self, state: &mut SearchState, start_idx: usize) -> bool {
        for i in start_idx..self.options.len() {
            let (_, ref constraints) = self.options[i];
            
            // Check if this option conflicts with already covered constraints
            if constraints.iter().any(|&c| state.covered[c]) {
                continue;
            }

            // Cover these constraints
            for &c in constraints {
                state.covered[c] = true;
            }
            state.stack.push((i, i + 1));
            return true;
        }
        false
    }

    fn backtrack_one(&self, state: &mut SearchState) -> bool {
        loop {
            let Some((opt_idx, _)) = state.stack.pop() else {
                return false; // No more to backtrack
            };

            // Uncover constraints from this option
            let (_, ref constraints) = self.options[opt_idx];
            for &c in constraints {
                state.covered[c] = false;
            }

            // Try next option at this level
            let next_start = opt_idx + 1;
            if next_start >= self.options.len() {
                continue; // No more options at this level
            }

            // Try extending from next option
            for i in next_start..self.options.len() {
                let (_, ref constraints) = self.options[i];
                
                // Check if this option conflicts with already covered constraints
                if constraints.iter().any(|&c| state.covered[c]) {
                    continue;
                }

                // Cover these constraints
                for &c in constraints {
                    state.covered[c] = true;
                }
                state.stack.push((i, i + 1));
                return true;
            }
            // Continue backtracking
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Choice {
        id: u32,
    }

    #[test]
    fn test_simple_exact_cover() {
        let mut solver = Solver::new(3);
        
        // Option 1 covers constraints {1, 2}
        solver.add_option(Choice { id: 1 }, &[1, 2]);
        
        // Option 2 covers constraint {3}
        solver.add_option(Choice { id: 2 }, &[3]);
        
        let solution = solver.next().unwrap();
        assert_eq!(solution.len(), 2);
        assert!(solution.contains(&Choice { id: 1 }));
        assert!(solution.contains(&Choice { id: 2 }));
    }

    #[test]
    fn test_no_solution() {
        let mut solver = Solver::new(3);
        
        // Option 1 covers {1, 2}
        solver.add_option(Choice { id: 1 }, &[1, 2]);
        
        // Option 2 also covers {1, 2} - conflicts with option 1
        solver.add_option(Choice { id: 2 }, &[1, 2]);
        
        // Constraint 3 is never covered
        let solution = solver.next();
        assert!(solution.is_none());
    }

    #[test]
    fn test_multiple_solutions() {
        let mut solver = Solver::new(2);
        
        // Two ways to cover both constraints
        solver.add_option(Choice { id: 1 }, &[1]);
        solver.add_option(Choice { id: 2 }, &[2]);
        solver.add_option(Choice { id: 3 }, &[1, 2]);
        
        let sol1 = solver.next().unwrap();
        assert_eq!(sol1.len(), 2);
        
        let sol2 = solver.next().unwrap();
        assert_eq!(sol2.len(), 1);
        assert_eq!(sol2[0].id, 3);
        
        assert!(solver.next().is_none());
    }
}
