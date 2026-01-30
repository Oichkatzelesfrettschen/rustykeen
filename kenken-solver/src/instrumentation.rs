#![doc = "Instrumentation and event callback system for solver visualization (requires `ui-instrumentation` feature)"]

use kenken_core::CellId;

/// Events emitted during solver execution for UI visualization and debugging.
///
/// When the `ui-instrumentation` feature is enabled, the solver emits these events
/// at key points in the search process. The events can be collected and replayed
/// for animated visualization, deduction step-by-step display, or performance analysis.
///
/// When the feature is disabled, all event emission code is compiled away (zero overhead).
#[derive(Debug, Clone)]
pub enum SolverEvent {
    /// A cell value has been assigned during search or propagation.
    ///
    /// Emitted when `domain[cell]` becomes a singleton (one possible value).
    /// This is the primary event for grid visualization updates.
    Assignment {
        /// The cell being assigned.
        cell: CellId,
        /// The value assigned (1..=n for n x n grid).
        value: u8,
        /// Depth in the search tree (0 for initial assignments, increases during backtracking).
        depth: u32,
        /// Whether this assignment came from deduction (true) or a search guess (false).
        deduced: bool,
    },

    /// Search has backtracked to an earlier point in the tree.
    ///
    /// Emitted when the solver abandons a search branch and reverts to a saved state.
    /// Used to visualize the branch that failed.
    Backtrack {
        /// The depth level being returned to.
        from_depth: u32,
        /// Number of assignments being undone.
        assignments_undone: u32,
        /// Human-readable reason for backtrack ("domain wipeout", "constraint violated", etc).
        reason: String,
    },

    /// Constraint propagation has occurred, reducing domains of affected cells.
    ///
    /// Emitted after any deduction rule fires and eliminates values from one or more domains.
    /// Used for ripple-effect visualization showing which cells were affected.
    Propagation {
        /// Cells whose domains were reduced in this propagation step.
        affected_cells: Vec<CellId>,
        /// Total number of values eliminated across all affected cells.
        values_eliminated: u32,
        /// Name of the deduction rule that fired ("row_unique", "col_unique", "cage_sum", etc).
        rule_name: String,
    },

    /// A deduction rule has successfully produced a new constraint.
    ///
    /// Emitted when constraint propagation applies a rule and learns something new.
    /// Used for step-by-step tutorial explanations.
    Deduction {
        /// Human-readable explanation of what was deduced.
        /// Example: "Row 3 constraint eliminated 5 from R3C2"
        explanation: String,
        /// Which cells were directly affected by this deduction.
        affected_cells: Vec<CellId>,
        /// The deduction tier that produced this (None/Easy/Normal/Hard).
        tier_name: String,
    },

    /// A conflict or constraint violation was detected.
    ///
    /// Emitted when the solver detects that a constraint cannot be satisfied,
    /// triggering backtracking. Used to visualize why backtracking occurred.
    Conflict {
        /// The conflicting cell (often the first to violate a constraint).
        cell: CellId,
        /// Human-readable description of the conflict.
        /// Example: "Row 3 already contains value 2"
        reason: String,
        /// Current search depth.
        depth: u32,
    },

    /// Search has branched and made a guess at a cell value.
    ///
    /// Emitted just before trying a candidate value during search.
    /// Used to mark branch points in the visualization.
    Branch {
        /// The cell being guessed.
        cell: CellId,
        /// The candidate value being tried.
        value: u8,
        /// Current depth in search tree.
        depth: u32,
        /// Total remaining candidates for this cell (for branching factor info).
        candidates_remaining: u32,
    },

    /// Solver has reached a complete solution.
    ///
    /// Emitted once when a valid solution is found.
    /// Used to finalize animations and display solved state.
    SolutionFound {
        /// Total assignments made during search.
        total_assignments: u64,
        /// Deepest backtracking level reached.
        max_depth: u32,
        /// Total search nodes explored.
        nodes_visited: u64,
    },

    /// Solver has determined no solution exists.
    ///
    /// Emitted when exhaustive search completes without finding a valid solution.
    NoSolution {
        /// Total assignments explored before giving up.
        total_assignments: u64,
        /// Deepest backtracking level reached.
        max_depth: u32,
        /// Total search nodes explored.
        nodes_visited: u64,
    },
}

impl SolverEvent {
    /// Returns a human-readable event type name for logging/display.
    pub fn event_type(&self) -> &'static str {
        match self {
            SolverEvent::Assignment { .. } => "Assignment",
            SolverEvent::Backtrack { .. } => "Backtrack",
            SolverEvent::Propagation { .. } => "Propagation",
            SolverEvent::Deduction { .. } => "Deduction",
            SolverEvent::Conflict { .. } => "Conflict",
            SolverEvent::Branch { .. } => "Branch",
            SolverEvent::SolutionFound { .. } => "SolutionFound",
            SolverEvent::NoSolution { .. } => "NoSolution",
        }
    }

    /// Returns the search depth associated with this event (if applicable).
    pub fn depth(&self) -> Option<u32> {
        match self {
            SolverEvent::Assignment { depth, .. } => Some(*depth),
            SolverEvent::Backtrack { from_depth, .. } => Some(*from_depth),
            SolverEvent::Conflict { depth, .. } => Some(*depth),
            SolverEvent::Branch { depth, .. } => Some(*depth),
            _ => None,
        }
    }
}

/// Callback trait for receiving solver events.
///
/// Implement this trait to collect events for visualization, logging, or analysis.
/// The callback is invoked synchronously from the solver's main loop, so keep
/// it fast to avoid impacting solver performance.
///
/// This trait is only used when the `ui-instrumentation` feature is enabled.
pub trait SolverEventCallback: Send {
    /// Called when a solver event occurs.
    ///
    /// # Performance
    /// Keep this method fast. Heavy computations will slow down the solver.
    /// Consider buffering events and processing them asynchronously in UI code.
    fn on_event(&mut self, event: SolverEvent);
}

/// Simple event collector that stores all events in a vector.
///
/// Useful for testing, replay, or post-mortem analysis.
#[derive(Debug, Default, Clone)]
pub struct EventCollector {
    /// Accumulated events in chronological order.
    pub events: Vec<SolverEvent>,
}

impl EventCollector {
    /// Creates a new empty event collector.
    pub fn new() -> Self {
        EventCollector::default()
    }

    /// Clear all accumulated events.
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Get the number of events collected.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if collector is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Iterator over collected events.
    pub fn iter(&self) -> impl Iterator<Item = &SolverEvent> {
        self.events.iter()
    }
}

impl SolverEventCallback for EventCollector {
    fn on_event(&mut self, event: SolverEvent) {
        self.events.push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_names() {
        let assignment = SolverEvent::Assignment {
            cell: CellId(0),
            value: 3,
            depth: 0,
            deduced: true,
        };
        assert_eq!(assignment.event_type(), "Assignment");

        let backtrack = SolverEvent::Backtrack {
            from_depth: 5,
            assignments_undone: 3,
            reason: "test".to_string(),
        };
        assert_eq!(backtrack.event_type(), "Backtrack");
    }

    #[test]
    fn event_depth() {
        let assignment = SolverEvent::Assignment {
            cell: CellId(0),
            value: 3,
            depth: 2,
            deduced: false,
        };
        assert_eq!(assignment.depth(), Some(2));

        let propagation = SolverEvent::Propagation {
            affected_cells: vec![],
            values_eliminated: 0,
            rule_name: "test".to_string(),
        };
        assert_eq!(propagation.depth(), None);
    }

    #[test]
    fn event_collector() {
        let mut collector = EventCollector::new();
        assert!(collector.is_empty());

        let event = SolverEvent::Assignment {
            cell: CellId(0),
            value: 1,
            depth: 0,
            deduced: true,
        };
        collector.on_event(event.clone());
        assert_eq!(collector.len(), 1);
        assert!(!collector.is_empty());

        let collected = collector.iter().next().unwrap();
        assert_eq!(collected.event_type(), "Assignment");

        collector.clear();
        assert!(collector.is_empty());
    }
}
