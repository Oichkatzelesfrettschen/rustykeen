#![doc = "Grid widget for displaying and interacting with KenKen puzzles"]

use gtk4::DrawingArea;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Custom GTK4 DrawingArea-based grid widget for KenKen puzzles
pub struct GridWidget {
    drawing_area: DrawingArea,
    state: Rc<RefCell<GridState>>,
}

struct GridState {
    width: u32,
    height: u32,
    cell_size: f64,
    selected_cell: Option<(u32, u32)>,
}

impl GridWidget {
    /// Create a new grid widget
    pub fn new(width: u32, height: u32) -> Self {
        let drawing_area = DrawingArea::new();
        let cell_size = 80.0; // pixels per cell

        // Set minimum size
        let min_width = (width as f64 * cell_size) as i32;
        let min_height = (height as f64 * cell_size) as i32;
        drawing_area.set_size_request(min_width, min_height);

        let state = Rc::new(RefCell::new(GridState {
            width,
            height,
            cell_size,
            selected_cell: None,
        }));

        let widget = Self {
            drawing_area,
            state,
        };

        widget.setup_drawing();
        widget
    }

    /// Get the GTK4 widget
    pub fn widget(&self) -> &DrawingArea {
        &self.drawing_area
    }

    /// Get selected cell
    pub fn selected_cell(&self) -> Option<(u32, u32)> {
        self.state.borrow().selected_cell
    }

    /// Set selected cell
    pub fn set_selected_cell(&mut self, row: u32, col: u32) {
        let mut state = self.state.borrow_mut();
        if row < state.height && col < state.width {
            state.selected_cell = Some((row, col));
            drop(state);
            self.drawing_area.queue_draw();
        }
    }

    fn setup_drawing(&self) {
        let state = Rc::clone(&self.state);

        self.drawing_area
            .set_draw_func(move |_area, context, _width, _height| {
                let s = state.borrow();
                Self::draw_grid(context, s.width, s.height, s.cell_size, s.selected_cell);
            });
    }

    fn draw_grid(
        context: &cairo::Context,
        width: u32,
        height: u32,
        cell_size: f64,
        _selected: Option<(u32, u32)>,
    ) {
        let bg_color = (0.95, 0.95, 0.95); // Light gray
        context.set_source_rgb(bg_color.0, bg_color.1, bg_color.2);
        context.paint().expect("Paint failed");

        // Draw cell grid
        context.set_line_width(1.0);
        context.set_source_rgb(0.7, 0.7, 0.7);

        for row in 0..=height {
            let y = row as f64 * cell_size;
            context.move_to(0.0, y);
            context.line_to(width as f64 * cell_size, y);
            context.stroke().expect("Stroke failed");
        }

        for col in 0..=width {
            let x = col as f64 * cell_size;
            context.move_to(x, 0.0);
            context.line_to(x, height as f64 * cell_size);
            context.stroke().expect("Stroke failed");
        }

        // Draw thicker cage borders (placeholder)
        context.set_line_width(2.0);
        context.set_source_rgb(0.2, 0.2, 0.2);

        // Draw outer border
        context.rectangle(
            0.0,
            0.0,
            width as f64 * cell_size,
            height as f64 * cell_size,
        );
        context.stroke().expect("Stroke failed");

        // Draw selected cell highlight
        if let Some((_row, _col)) = _selected {
            context.set_source_rgba(0.3, 0.5, 1.0, 0.3);
            // Placeholder: would draw selection highlight here
        }
    }
}

impl Default for GridWidget {
    fn default() -> Self {
        Self::new(4, 4)
    }
}
