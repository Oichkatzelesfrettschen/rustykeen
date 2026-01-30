//! Internal Dancing Links (DLX) exact cover solver.
//!
//! Simplified implementation of Algorithm X with Dancing Links by Donald Knuth.
//! Replaces the external `dlx-rs` dependency with ~200 LOC of internal code.
//!
//! ## Algorithm Overview
//!
//! Dancing Links solves exact cover problems efficiently:
//! - Given a matrix of 0s and 1s, find subsets of rows where each column has exactly one 1
//! - Uses circular doubly-linked lists for O(1) cover/uncover operations
//! - Backtracking search with column selection heuristic
//!
//! ## References
//!
//! - Knuth, Donald E. "Dancing links" (2000)

#![allow(dead_code)]

/// A node in the Dancing Links sparse matrix.
#[derive(Clone)]
struct Node {
    down: usize,
    up: usize,
    right: usize,
    left: usize,
    column: usize,
}

/// Column header metadata.
#[derive(Clone)]
struct Column {
    size: usize,
    node: usize,
}

/// Dancing Links exact cover solver.
pub struct Solver<T> {
    nodes: Vec<Node>,
    columns: Vec<Column>,
    options: Vec<T>,
    option_nodes: Vec<usize>,
    root: usize,
    solutions: Vec<Vec<usize>>,
    solution_index: usize,
}

impl<T: Clone> Solver<T> {
    /// Create a new solver for `num_constraints` columns.
    pub fn new(num_constraints: usize) -> Self {
        let num_nodes = 1 + num_constraints;
        let mut nodes = Vec::with_capacity(num_nodes);
        let mut columns = Vec::with_capacity(num_constraints);

        // Root node (index 0)
        nodes.push(Node {
            down: 0,
            up: 0,
            right: if num_constraints > 0 { 1 } else { 0 },
            left: num_constraints,
            column: 0,
        });

        // Column headers (indices 1..=num_constraints)
        for i in 1..=num_constraints {
            let next = if i < num_constraints { i + 1 } else { 0 };
            let prev = i - 1;
            nodes.push(Node {
                down: i,
                up: i,
                right: next,
                left: prev,
                column: i,
            });
            columns.push(Column { size: 0, node: i });
        }

        Solver {
            nodes,
            columns,
            options: Vec::new(),
            option_nodes: Vec::new(),
            root: 0,
            solutions: Vec::new(),
            solution_index: 0,
        }
    }

    /// Add an option (row) that covers the given constraints (columns).
    pub fn add_option(&mut self, data: T, constraints: &[usize]) {
        if constraints.is_empty() {
            return;
        }

        self.options.push(data);
        let first_node = self.nodes.len();
        self.option_nodes.push(first_node);

        for (i, &col_idx) in constraints.iter().enumerate() {
            assert!(col_idx > 0 && col_idx <= self.columns.len());

            let col = &mut self.columns[col_idx - 1];
            let col_node = col.node;
            let node_idx = self.nodes.len();
            let up = self.nodes[col_node].up;
            let left = if i > 0 {
                node_idx - 1
            } else {
                first_node + constraints.len() - 1
            };
            let right = if i < constraints.len() - 1 {
                node_idx + 1
            } else {
                first_node
            };

            self.nodes.push(Node {
                down: col_node,
                up,
                right,
                left,
                column: col_idx,
            });

            self.nodes[col_node].up = node_idx;
            self.nodes[up].down = node_idx;
            col.size += 1;
        }
    }

    /// Find the next solution.
    pub fn next(&mut self) -> Option<Vec<T>> {
        // Find all solutions on first call
        if self.solution_index == 0 && self.solutions.is_empty() {
            let mut partial = Vec::new();
            self.search(&mut partial);
        }

        if self.solution_index < self.solutions.len() {
            let sol = &self.solutions[self.solution_index];
            self.solution_index += 1;
            Some(sol.iter().map(|&i| self.options[i].clone()).collect())
        } else {
            None
        }
    }

    fn search(&mut self, partial: &mut Vec<usize>) {
        // Base case: all columns covered
        if self.nodes[self.root].right == self.root {
            self.solutions.push(partial.clone());
            return;
        }

        // Choose column with minimum size
        let col = self.choose_column();
        if self.columns[col - 1].size == 0 {
            return; // No solution
        }

        self.cover(col);

        // Try each row in this column
        let mut row = self.nodes[col].down;
        while row != col {
            partial.push(self.option_index(row));

            // Cover other columns in this row
            let mut node = self.nodes[row].right;
            while node != row {
                self.cover(self.nodes[node].column);
                node = self.nodes[node].right;
            }

            // Recurse
            self.search(partial);

            // Uncover columns
            node = self.nodes[row].left;
            while node != row {
                self.uncover(self.nodes[node].column);
                node = self.nodes[node].left;
            }

            partial.pop();
            row = self.nodes[row].down;
        }

        self.uncover(col);
    }

    fn choose_column(&self) -> usize {
        let mut min_col = self.nodes[self.root].right;
        let mut min_size = self.columns[min_col - 1].size;

        let mut col = self.nodes[min_col].right;
        while col != self.root {
            let size = self.columns[col - 1].size;
            if size < min_size {
                min_col = col;
                min_size = size;
            }
            col = self.nodes[col].right;
        }
        min_col
    }

    fn cover(&mut self, col: usize) {
        let left = self.nodes[col].left;
        let right = self.nodes[col].right;
        self.nodes[left].right = right;
        self.nodes[right].left = left;

        let mut row = self.nodes[col].down;
        while row != col {
            let mut node = self.nodes[row].right;
            while node != row {
                let up = self.nodes[node].up;
                let down = self.nodes[node].down;
                self.nodes[up].down = down;
                self.nodes[down].up = up;
                self.columns[self.nodes[node].column - 1].size -= 1;
                node = self.nodes[node].right;
            }
            row = self.nodes[row].down;
        }
    }

    fn uncover(&mut self, col: usize) {
        let mut row = self.nodes[col].up;
        while row != col {
            let mut node = self.nodes[row].left;
            while node != row {
                let up = self.nodes[node].up;
                let down = self.nodes[node].down;
                self.nodes[up].down = node;
                self.nodes[down].up = node;
                self.columns[self.nodes[node].column - 1].size += 1;
                node = self.nodes[node].left;
            }
            row = self.nodes[row].up;
        }

        let left = self.nodes[col].left;
        let right = self.nodes[col].right;
        self.nodes[left].right = col;
        self.nodes[right].left = col;
    }

    fn option_index(&self, node: usize) -> usize {
        match self.option_nodes.binary_search(&node) {
            Ok(i) => i,
            Err(i) => i - 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_problem_solved_immediately() {
        let mut solver: Solver<i32> = Solver::new(0);
        assert!(solver.next().is_some());
        assert!(solver.next().is_none());
    }

    #[test]
    fn single_option_covers_all() {
        let mut solver = Solver::new(3);
        solver.add_option(1, &[1, 2, 3]);
        let solution = solver.next().unwrap();
        assert_eq!(solution, vec![1]);
        assert!(solver.next().is_none());
    }

    #[test]
    fn two_options_exact_cover() {
        let mut solver = Solver::new(3);
        solver.add_option(1, &[1, 2]);
        solver.add_option(2, &[3]);
        let solution = solver.next().unwrap();
        assert_eq!(solution.len(), 2);
        assert!(solution.contains(&1));
        assert!(solution.contains(&2));
    }

    #[test]
    fn no_solution_when_column_uncovered() {
        let mut solver = Solver::new(3);
        solver.add_option(1, &[1, 2]);
        assert!(solver.next().is_none());
    }

    #[test]
    fn multiple_solutions_enumeration() {
        let mut solver = Solver::new(2);
        solver.add_option(1, &[1]);
        solver.add_option(2, &[2]);

        let sol1 = solver.next();
        assert!(sol1.is_some());
        let sol2 = solver.next();
        assert!(sol2.is_none());
    }
}
