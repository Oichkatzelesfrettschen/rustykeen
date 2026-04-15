#![doc = "Control panel with solve button and solver statistics"]

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation};
use kenken_core::Puzzle;
use kenken_solver::{Ruleset, SolveStats, solve_one_with_stats_dispatched};
use std::cell::RefCell;
use std::rc::Rc;

/// Solver result with solution and statistics
#[derive(Clone)]
pub struct SolveResult {
    pub solution: Vec<u8>,
    pub n: u8,
    pub stats: SolveStats,
}

/// Control panel widget
pub struct ControlPanel {
    container: GtkBox,
    solve_button: Button,
    status_label: Label,
    stats_label: Label,
    result: Rc<RefCell<Option<SolveResult>>>,
}

impl ControlPanel {
    /// Create a new control panel
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 8);
        container.set_margin_top(8);
        container.set_margin_bottom(8);
        container.set_margin_start(8);
        container.set_margin_end(8);

        // Button section
        let button_box = GtkBox::new(Orientation::Horizontal, 4);
        let solve_button = Button::with_label("Solve Puzzle");
        button_box.append(&solve_button);
        container.append(&button_box);

        // Status label
        let status_label = Label::new(Some("Ready"));
        status_label.set_halign(gtk4::Align::Start);
        container.append(&status_label);

        // Statistics label
        let stats_label = Label::new(None);
        stats_label.set_halign(gtk4::Align::Start);
        stats_label.set_wrap(true);
        container.append(&stats_label);

        Self {
            container,
            solve_button,
            status_label,
            stats_label,
            result: Rc::new(RefCell::new(None)),
        }
    }

    /// Get the container widget
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Get the solve button for connecting signals
    pub fn solve_button(&self) -> &Button {
        &self.solve_button
    }

    /// Solve a puzzle with baseline KenKen ruleset
    pub fn solve_puzzle(&self, puzzle: &Puzzle) -> Option<SolveResult> {
        self.status_label.set_label("Solving...");

        let rules = Ruleset::keen_baseline();
        match solve_one_with_stats_dispatched(puzzle, rules) {
            Ok((Some(solution), stats)) => {
                let result = SolveResult {
                    solution: solution.grid.clone(),
                    n: puzzle.n,
                    stats,
                };

                // Update status and stats display
                self.status_label.set_label("Puzzle solved!");

                let stats_text = format!(
                    "Assignments: {} | Nodes: {} | Max depth: {} | Backtracked: {}",
                    stats.assignments, stats.nodes_visited, stats.max_depth, stats.backtracked
                );
                self.stats_label.set_label(&stats_text);

                *self.result.borrow_mut() = Some(result.clone());
                Some(result)
            }
            Ok((None, _)) => {
                self.status_label.set_label("No solution found");
                None
            }
            Err(e) => {
                self.status_label.set_label(&format!("Error: {}", e));
                None
            }
        }
    }

    /// Get the last solve result
    pub fn last_result(&self) -> Option<SolveResult> {
        self.result.borrow().clone()
    }
}

impl Default for ControlPanel {
    fn default() -> Self {
        Self::new()
    }
}
