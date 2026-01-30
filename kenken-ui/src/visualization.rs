#![doc = "Grid rendering and visualization for solver backtracking animation"]

pub mod ripple;

use cairo::Context;
use kenken_core::CellId;
pub use ripple::{Ripple, RippleAnimator, RippleBurst};

/// Color palette for visualizing solver state.
#[derive(Debug, Clone)]
pub struct VisualizationColors {
    /// Background color for cells (RGB 0-1)
    pub background: (f64, f64, f64),
    /// Color for successfully assigned cells (green)
    pub assigned_success: (f64, f64, f64),
    /// Color for cells being explored (yellow/orange)
    pub exploring: (f64, f64, f64),
    /// Color for cells being backtracked (red)
    pub backtracked: (f64, f64, f64),
    /// Color for text/grid lines (dark)
    pub text: (f64, f64, f64),
    /// Color for selection highlight (blue)
    pub selected: (f64, f64, f64),
    /// Background for conflicted cells
    pub conflict: (f64, f64, f64),
}

impl Default for VisualizationColors {
    fn default() -> Self {
        VisualizationColors {
            background: (0.95, 0.95, 0.95),
            assigned_success: (0.1, 0.8, 0.1),
            exploring: (1.0, 0.8, 0.0),
            backtracked: (1.0, 0.2, 0.2),
            text: (0.2, 0.2, 0.2),
            selected: (0.3, 0.5, 1.0),
            conflict: (1.0, 0.5, 0.5),
        }
    }
}

/// Animation frame data for a single cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    /// Cell is unassigned with no special state
    Empty,
    /// Cell has been successfully assigned (deduction or search)
    Assigned,
    /// Cell is being explored in current search branch
    Exploring { depth: u32 },
    /// Cell was assigned but then backtracked
    Backtracked,
    /// Cell violates a constraint
    Conflict,
    /// Cell is selected by user
    Selected,
}

/// Animation frame for a moment in time during solver execution.
#[derive(Debug, Clone)]
pub struct AnimationFrame {
    /// State of each cell (indexed by CellId.0)
    pub cell_states: Vec<CellState>,
    /// Grid size (n)
    pub grid_size: u8,
    /// Current depth in search tree
    pub search_depth: u32,
    /// Number of assignments made so far
    pub assignments: u64,
}

impl AnimationFrame {
    /// Create a new frame for an n x n grid.
    pub fn new(grid_size: u8) -> Self {
        let size = (grid_size as usize).pow(2);
        AnimationFrame {
            cell_states: vec![CellState::Empty; size],
            grid_size,
            search_depth: 0,
            assignments: 0,
        }
    }

    /// Set state of a cell.
    pub fn set_cell_state(&mut self, cell: CellId, state: CellState) {
        if (cell.0 as usize) < self.cell_states.len() {
            self.cell_states[cell.0 as usize] = state;
        }
    }

    /// Get state of a cell.
    pub fn get_cell_state(&self, cell: CellId) -> CellState {
        if (cell.0 as usize) < self.cell_states.len() {
            self.cell_states[cell.0 as usize]
        } else {
            CellState::Empty
        }
    }
}

/// Grid renderer with backtracking visualization support.
pub struct GridRenderer {
    grid_size: u8,
    cell_size: f64,
    padding: f64,
    colors: VisualizationColors,
}

impl GridRenderer {
    /// Create a new grid renderer.
    pub fn new(grid_size: u8, cell_size: f64) -> Self {
        GridRenderer {
            grid_size,
            cell_size,
            padding: 2.0,
            colors: VisualizationColors::default(),
        }
    }

    /// Set custom colors.
    pub fn set_colors(&mut self, colors: VisualizationColors) {
        self.colors = colors;
    }

    /// Draw a grid frame with backtracking visualization.
    ///
    /// Renders the grid with cells colored according to their current state.
    /// Shows search depth and assignment count.
    pub fn draw_frame(&self, ctx: &Context, frame: &AnimationFrame) -> Result<(), cairo::Error> {
        let grid_size = self.grid_size as usize;

        // Draw grid background
        ctx.set_source_rgb(self.colors.background.0, self.colors.background.1, self.colors.background.2);
        ctx.rectangle(0.0, 0.0, (grid_size as f64) * self.cell_size, (grid_size as f64) * self.cell_size);
        ctx.fill()?;

        // Draw cells with state-based coloring
        for row in 0..grid_size {
            for col in 0..grid_size {
                let cell_id = CellId((row * grid_size + col) as u16);
                self.draw_cell(ctx, row, col, frame.get_cell_state(cell_id))?;
            }
        }

        // Draw grid lines
        ctx.set_source_rgb(self.colors.text.0, self.colors.text.1, self.colors.text.2);
        ctx.set_line_width(1.0);

        for i in 0..=grid_size {
            let pos = (i as f64) * self.cell_size;
            // Horizontal
            ctx.move_to(0.0, pos);
            ctx.line_to((grid_size as f64) * self.cell_size, pos);
            ctx.stroke()?;
            // Vertical
            ctx.move_to(pos, 0.0);
            ctx.line_to(pos, (grid_size as f64) * self.cell_size);
            ctx.stroke()?;
        }

        Ok(())
    }

    /// Draw a single cell with visualization state.
    fn draw_cell(&self, ctx: &Context, row: usize, col: usize, state: CellState) -> Result<(), cairo::Error> {
        let x = (col as f64) * self.cell_size;
        let y = (row as f64) * self.cell_size;
        let inner_size = self.cell_size - (2.0 * self.padding);

        // Choose color based on state
        let (r, g, b) = match state {
            CellState::Empty => self.colors.background,
            CellState::Assigned => self.colors.assigned_success,
            CellState::Exploring { .. } => self.colors.exploring,
            CellState::Backtracked => self.colors.backtracked,
            CellState::Conflict => self.colors.conflict,
            CellState::Selected => self.colors.selected,
        };

        // Draw cell background
        ctx.set_source_rgb(r, g, b);
        ctx.rectangle(x + self.padding, y + self.padding, inner_size, inner_size);
        ctx.fill()?;

        // Draw selection highlight if selected
        if state == CellState::Selected {
            ctx.set_source_rgba(self.colors.selected.0, self.colors.selected.1, self.colors.selected.2, 0.3);
            ctx.set_line_width(3.0);
            ctx.rectangle(x + self.padding, y + self.padding, inner_size, inner_size);
            ctx.stroke()?;
        }

        Ok(())
    }

    /// Draw a ripple effect emanating from a cell (for propagation visualization).
    pub fn draw_ripple(&self, ctx: &Context, cell: CellId, intensity: f64, grid_size: u8) -> Result<(), cairo::Error> {
        let grid_usize = grid_size as usize;
        let row = (cell.0 as usize) / grid_usize;
        let col = (cell.0 as usize) % grid_usize;

        let center_x = (col as f64 + 0.5) * self.cell_size;
        let center_y = (row as f64 + 0.5) * self.cell_size;

        // Draw concentric ripples with decreasing opacity
        for i in 1..=5 {
            let radius = (i as f64) * 15.0 * intensity;
            let alpha = 0.3 * (1.0 - (i as f64 / 5.0));

            ctx.set_source_rgba(0.3, 0.8, 0.3, alpha);
            ctx.set_line_width(2.0);
            ctx.arc(center_x, center_y, radius, 0.0, 2.0 * std::f64::consts::PI);
            ctx.stroke()?;
        }

        Ok(())
    }

    /// Draw depth indicator showing search tree depth.
    pub fn draw_depth_indicator(&self, ctx: &Context, depth: u32, max_depth: u32, canvas_width: f64) -> Result<(), cairo::Error> {
        let y_pos = (self.grid_size as f64 + 1.0) * self.cell_size + 20.0;
        let bar_width = canvas_width - 40.0;
        let bar_height = 20.0;

        // Draw background bar
        ctx.set_source_rgb(0.9, 0.9, 0.9);
        ctx.rectangle(20.0, y_pos, bar_width, bar_height);
        ctx.fill()?;

        // Draw progress bar
        let progress = if max_depth > 0 {
            (depth as f64) / (max_depth as f64)
        } else {
            0.0
        };

        ctx.set_source_rgb(0.3, 0.5, 1.0);
        ctx.rectangle(20.0, y_pos, bar_width * progress, bar_height);
        ctx.fill()?;

        // Draw text
        let text = format!("Depth: {}/{}", depth, max_depth);
        ctx.set_source_rgb(0.2, 0.2, 0.2);
        ctx.set_font_size(12.0);
        ctx.move_to(30.0, y_pos + 15.0);
        ctx.show_text(&text)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_frame_creation() {
        let frame = AnimationFrame::new(4);
        assert_eq!(frame.grid_size, 4);
        assert_eq!(frame.cell_states.len(), 16);
        assert_eq!(frame.search_depth, 0);
        assert_eq!(frame.assignments, 0);
    }

    #[test]
    fn animation_frame_cell_state() {
        let mut frame = AnimationFrame::new(4);
        let cell = CellId(0);

        assert_eq!(frame.get_cell_state(cell), CellState::Empty);

        frame.set_cell_state(cell, CellState::Assigned);
        assert_eq!(frame.get_cell_state(cell), CellState::Assigned);

        frame.set_cell_state(cell, CellState::Exploring { depth: 3 });
        assert_eq!(frame.get_cell_state(cell), CellState::Exploring { depth: 3 });
    }

    #[test]
    fn visualization_colors_default() {
        let colors = VisualizationColors::default();
        assert!(colors.background.0 > 0.9);
        assert!(colors.assigned_success.1 > 0.7);
        assert!(colors.backtracked.0 > 0.9);
    }

    #[test]
    fn grid_renderer_creation() {
        let renderer = GridRenderer::new(4, 80.0);
        assert_eq!(renderer.grid_size, 4);
        assert_eq!(renderer.cell_size, 80.0);
    }
}
