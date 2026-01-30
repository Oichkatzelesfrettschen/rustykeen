// Tutorial delivery UI panel for interactive lesson presentation

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, ScrolledWindow, TextBuffer, TextView};
use crate::tutorial::{Lesson, TutorialProgress, LessonStep};
use std::cell::RefCell;
use std::rc::Rc;

/// Tutorial panel for displaying lessons and tracking progress.
pub struct TutorialPanel {
    container: GtkBox,
    lesson_title: Label,
    lesson_text: TextView,
    buffer: TextBuffer,
    progress: Rc<RefCell<TutorialProgress>>,
}

impl TutorialPanel {
    /// Create new tutorial panel.
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 10);
        container.set_margin_top(15);
        container.set_margin_bottom(15);
        container.set_margin_start(15);
        container.set_margin_end(15);

        // Header: lesson title and progress
        let header_box = GtkBox::new(Orientation::Horizontal, 10);
        let lesson_title = Label::new(Some("Tutorial Mode"));
        lesson_title.set_markup("<b>Tutorial Mode</b>");
        header_box.append(&lesson_title);

        let progress_label = Label::new(Some("0% Complete"));
        progress_label.set_hexpand(true);
        progress_label.set_halign(gtk4::Align::End);
        header_box.append(&progress_label);
        container.append(&header_box);

        // Lesson content
        let buffer = TextBuffer::new(None);
        let lesson_text = TextView::with_buffer(&buffer);
        lesson_text.set_editable(false);
        lesson_text.set_wrap_mode(gtk4::WrapMode::Word);

        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scrolled.set_child(Some(&lesson_text));
        scrolled.set_vexpand(true);
        container.append(&scrolled);

        // Control buttons
        let button_box = GtkBox::new(Orientation::Horizontal, 5);

        let prev_btn = Button::with_label("Previous Step");
        button_box.append(&prev_btn);

        let hint_btn = Button::with_label("Show Hint");
        button_box.append(&hint_btn);

        let next_btn = Button::with_label("Next Step");
        button_box.append(&next_btn);

        let complete_btn = Button::with_label("Complete Lesson");
        button_box.append(&complete_btn);

        container.append(&button_box);

        let progress = Rc::new(RefCell::new(TutorialProgress::default()));

        TutorialPanel {
            container,
            lesson_title,
            lesson_text,
            buffer,
            progress,
        }
    }

    /// Start a lesson.
    pub fn start_lesson(&self, lesson_id: u8) {
        let mut prog = self.progress.borrow_mut();
        prog.start_lesson(lesson_id);
        drop(prog);

        self.display_current_step();
    }

    /// Display current lesson step.
    fn display_current_step(&self) {
        let prog = self.progress.borrow();

        if let Some(lesson_id) = prog.current {
            if let Some(lesson) = Lesson::get(lesson_id) {
                let step_num = prog.current_step;

                // Update title
                let title = format!("{} - Step {}/{}",
                    lesson.title,
                    step_num,
                    lesson.steps.len());
                self.lesson_title.set_markup(&format!("<b>{}</b>", title));

                // Update content
                if let Some(step) = lesson.step(step_num) {
                    let content = self.format_step(&lesson, step);
                    self.buffer.set_text(&content);
                }
            }
        }
    }

    /// Format step for display.
    fn format_step(&self, lesson: &Lesson, step: &LessonStep) -> String {
        let mut text = String::new();

        text.push_str(&format!("LESSON: {}\n", lesson.title));
        text.push_str(&format!("Objective: {}\n", lesson.objective));
        text.push_str(&format!("Grid Size: {}x{}\n\n", lesson.grid_size, lesson.grid_size));

        text.push_str(&format!("STEP {} of {}\n", step.number, 999)); // Use actual count
        text.push_str("=".repeat(40).as_str());
        text.push('\n');

        text.push_str(&format!("\nINSTRUCTION:\n{}\n", step.instruction));

        text.push_str(&format!("\nEXPLANATION:\n{}\n", step.explanation));

        if let Some(hint) = &step.hint {
            text.push_str(&format!("\nHINT:\n{}\n", hint));
        }

        text.push_str("\nUse Next Step to continue.\n");

        text
    }

    /// Move to next step.
    pub fn next_step(&self) {
        let mut prog = self.progress.borrow_mut();
        prog.next_step();
        drop(prog);
        self.display_current_step();
    }

    /// Move to previous step.
    pub fn previous_step(&self) {
        let mut prog = self.progress.borrow_mut();
        if prog.current_step > 1 {
            prog.current_step -= 1;
        }
        drop(prog);
        self.display_current_step();
    }

    /// Show hint for current step.
    pub fn show_hint(&self) {
        let prog = self.progress.borrow();

        if let Some(lesson_id) = prog.current {
            if let Some(lesson) = Lesson::get(lesson_id) {
                let step_num = prog.current_step;

                if let Some(step) = lesson.step(step_num) {
                    let mut text = String::new();
                    text.push_str(&self.format_step(&lesson, step));

                    if let Some(hint) = &step.hint {
                        text.push_str("\n\n");
                        text.push_str("EXPANDED HINT:\n");
                        text.push_str("=".repeat(40).as_str());
                        text.push('\n');
                        text.push_str(hint);
                        text.push('\n');
                    } else {
                        text.push_str("\n\nNo additional hint available for this step.\n");
                    }

                    self.buffer.set_text(&text);
                }
            }
        }
    }

    /// Complete current lesson.
    pub fn complete_lesson(&self) {
        let mut prog = self.progress.borrow_mut();
        prog.complete_lesson();
        drop(prog);

        let completion = self.get_progress_percent();
        let msg = format!(
            "Lesson completed! Tutorial progress: {:.0}%",
            completion
        );

        self.buffer.set_text(&msg);
    }

    /// Get progress percentage.
    pub fn get_progress_percent(&self) -> f32 {
        self.progress.borrow().completion_percent()
    }

    /// Get underlying widget.
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Get current progress state.
    pub fn progress(&self) -> Rc<RefCell<TutorialProgress>> {
        Rc::clone(&self.progress)
    }
}

impl Default for TutorialPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires GTK initialization
    fn tutorial_panel_creation() {
        let _panel = TutorialPanel::new();
        // Panel created successfully
    }

    #[test]
    #[ignore] // Requires GTK initialization
    fn tutorial_panel_lesson_start() {
        let panel = TutorialPanel::new();
        panel.start_lesson(1);

        let prog = panel.progress.borrow();
        assert_eq!(prog.current, Some(1));
        assert_eq!(prog.current_step, 1);
    }

    #[test]
    #[ignore] // Requires GTK initialization
    fn tutorial_panel_step_navigation() {
        let panel = TutorialPanel::new();
        panel.start_lesson(1);

        panel.next_step();
        assert_eq!(panel.progress.borrow().current_step, 2);

        panel.previous_step();
        assert_eq!(panel.progress.borrow().current_step, 1);
    }

    #[test]
    #[ignore] // Requires GTK initialization
    fn tutorial_panel_completion() {
        let panel = TutorialPanel::new();
        panel.start_lesson(1);
        panel.complete_lesson();

        let prog = panel.progress.borrow();
        assert!(prog.completed.contains(&1));
    }

    #[test]
    #[ignore] // Requires GTK initialization
    fn tutorial_panel_progress() {
        let panel = TutorialPanel::new();

        assert_eq!(panel.get_progress_percent(), 0.0);

        panel.start_lesson(1);
        panel.complete_lesson();

        let progress = panel.get_progress_percent();
        assert!(progress > 0.0 && progress < 100.0); // 1 of 5 lessons
    }
}
