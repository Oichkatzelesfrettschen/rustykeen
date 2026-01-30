#![doc = "Event timeline widget for displaying solver deduction history"]

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, ScrolledWindow, TextBuffer, TextView};
use kenken_solver::SolverEvent;
use std::cell::RefCell;
use std::rc::Rc;

/// Timeline panel showing solver events in chronological order.
///
/// Displays:
/// - Event type and human-readable description
/// - Affected cells
/// - Deduction tier or search depth
/// - Click to highlight affected cells in grid
pub struct EventTimelineWidget {
    container: GtkBox,
    text_view: TextView,
    buffer: TextBuffer,
    events: Rc<RefCell<Vec<(usize, SolverEvent)>>>, // (index, event)
}

impl EventTimelineWidget {
    /// Create a new event timeline widget.
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 5);
        container.set_margin_top(10);
        container.set_margin_bottom(10);
        container.set_margin_start(10);
        container.set_margin_end(10);

        // Title
        let title = Label::new(Some("Deduction Timeline"));
        title.set_markup("<b>Deduction Timeline</b>");
        container.append(&title);

        // Text view for timeline
        let buffer = TextBuffer::new(None);
        let text_view = TextView::with_buffer(&buffer);
        text_view.set_editable(false);
        text_view.set_wrap_mode(gtk4::WrapMode::Word);

        // Scrolled window
        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scrolled.set_child(Some(&text_view));
        scrolled.set_vexpand(true);

        container.append(&scrolled);

        // Control buttons
        let button_box = GtkBox::new(Orientation::Horizontal, 5);

        let clear_btn = Button::with_label("Clear");
        button_box.append(&clear_btn);

        let export_btn = Button::with_label("Export");
        button_box.append(&export_btn);

        container.append(&button_box);

        let events = Rc::new(RefCell::new(Vec::new()));

        EventTimelineWidget {
            container,
            text_view,
            buffer,
            events,
        }
    }

    /// Add an event to the timeline.
    pub fn add_event(&self, event: SolverEvent) {
        let mut events = self.events.borrow_mut();
        let idx = events.len();
        events.push((idx, event.clone()));
        drop(events);

        let (event_type, description, details) = self.format_event(&event, idx);

        let line = format!("[{}] {} - {}\n    {}\n", idx, event_type, description, details);

        let mut end_iter = self.buffer.end_iter();
        self.buffer.insert(&mut end_iter, &line);
    }

    /// Clear all events.
    pub fn clear(&self) {
        self.buffer.delete(&mut self.buffer.start_iter(), &mut self.buffer.end_iter());
        self.events.borrow_mut().clear();
    }

    /// Get timeline as text for export.
    pub fn export_text(&self) -> String {
        let events = self.events.borrow();
        let mut text = String::from("Solver Event Timeline\n");
        text.push_str("====================\n\n");

        for (idx, event) in events.iter() {
            let (etype, desc, details) = self.format_event(event, *idx);
            text.push_str(&format!("[{}] {} - {}\n    {}\n", idx, etype, desc, details));
        }

        text
    }

    /// Get the underlying GTK widget.
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Get the text view for direct manipulation if needed.
    pub fn text_view(&self) -> &TextView {
        &self.text_view
    }

    /// Format an event for display.
    fn format_event(&self, event: &SolverEvent, _idx: usize) -> (String, String, String) {
        let etype = event.event_type().to_string();

        let (description, details) = match event {
            SolverEvent::Assignment {
                cell,
                value,
                depth,
                deduced,
            } => {
                let desc = format!(
                    "Cell {} assigned value {} ({})",
                    cell.0,
                    value,
                    if *deduced { "deduced" } else { "guessed" }
                );
                let det = format!("Depth: {}", depth);
                (desc, det)
            }
            SolverEvent::Backtrack {
                from_depth,
                assignments_undone,
                reason,
            } => {
                let desc = format!("Backtracked from depth {}", from_depth);
                let det = format!("{} assignments undone: {}", assignments_undone, reason);
                (desc, det)
            }
            SolverEvent::Propagation {
                affected_cells,
                values_eliminated,
                rule_name,
            } => {
                let desc = format!(
                    "Propagation: {} rule affected {} cells",
                    rule_name,
                    affected_cells.len()
                );
                let det = format!("{} values eliminated", values_eliminated);
                (desc, det)
            }
            SolverEvent::Deduction {
                explanation,
                affected_cells,
                tier_name,
            } => {
                let desc = explanation.clone();
                let det = format!("Tier: {}, {} cells", tier_name, affected_cells.len());
                (desc, det)
            }
            SolverEvent::Conflict { cell, reason, depth } => {
                let desc = format!("Conflict at cell {}: {}", cell.0, reason);
                let det = format!("Depth: {}", depth);
                (desc, det)
            }
            SolverEvent::Branch {
                cell,
                value,
                depth,
                candidates_remaining,
            } => {
                let desc = format!("Branch: trying {} for cell {}", value, cell.0);
                let det = format!("Depth: {}, {} candidates left", depth, candidates_remaining);
                (desc, det)
            }
            SolverEvent::SolutionFound {
                total_assignments,
                max_depth,
                nodes_visited,
            } => {
                let desc = "Solution found!".to_string();
                let det = format!(
                    "Assignments: {}, Max depth: {}, Nodes: {}",
                    total_assignments, max_depth, nodes_visited
                );
                (desc, det)
            }
            SolverEvent::NoSolution {
                total_assignments,
                max_depth,
                nodes_visited,
            } => {
                let desc = "No solution exists (exhaustive search complete)".to_string();
                let det = format!(
                    "Assignments: {}, Max depth: {}, Nodes: {}",
                    total_assignments, max_depth, nodes_visited
                );
                (desc, det)
            }
        };

        (etype, description, details)
    }
}

impl Default for EventTimelineWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kenken_core::CellId;

    #[test]
    #[ignore] // Requires GTK initialization in test harness
    fn timeline_creation() {
        let timeline = EventTimelineWidget::new();
        assert_eq!(timeline.events.borrow().len(), 0);
    }

    #[test]
    #[ignore] // Requires GTK initialization in test harness
    fn timeline_add_event() {
        let timeline = EventTimelineWidget::new();

        let event = SolverEvent::Assignment {
            cell: CellId(0),
            value: 1,
            depth: 0,
            deduced: true,
        };

        timeline.add_event(event);
        assert_eq!(timeline.events.borrow().len(), 1);
    }

    #[test]
    #[ignore] // Requires GTK initialization in test harness
    fn timeline_export_text() {
        let timeline = EventTimelineWidget::new();

        let event1 = SolverEvent::Assignment {
            cell: CellId(0),
            value: 1,
            depth: 0,
            deduced: true,
        };

        let event2 = SolverEvent::SolutionFound {
            total_assignments: 16,
            max_depth: 0,
            nodes_visited: 16,
        };

        timeline.add_event(event1);
        timeline.add_event(event2);

        let text = timeline.export_text();
        assert!(text.contains("Solver Event Timeline"));
        assert!(text.contains("Assignment"));
        assert!(text.contains("SolutionFound"));
    }

    #[test]
    #[ignore] // Requires GTK initialization in test harness
    fn timeline_clear() {
        let timeline = EventTimelineWidget::new();

        let event = SolverEvent::Assignment {
            cell: CellId(0),
            value: 1,
            depth: 0,
            deduced: true,
        };

        timeline.add_event(event);
        assert_eq!(timeline.events.borrow().len(), 1);

        timeline.clear();
        assert_eq!(timeline.events.borrow().len(), 0);
    }
}
