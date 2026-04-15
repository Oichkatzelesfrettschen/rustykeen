use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::puzzle::{Cage, Puzzle};
use kenken_core::rules::{Op, Ruleset};
use kenken_solver::{
    DeductionTier, count_solutions_up_to_with_deductions, solve_one_with_deductions,
};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::{MouseButton, MouseWheelDirection};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::{FullscreenType, Window};

const DEFAULT_N: u8 = 2;
const DEFAULT_DESC: &str = "b__,a3a3";
const MIN_SCALE: f32 = 0.5;
const MAX_SCALE: f32 = 2.5;

#[derive(Clone)]
struct AppState {
    puzzle: Puzzle,
    rules: Ruleset,
    tier: DeductionTier,
    values: Vec<u8>,
    selected: Option<usize>,
    ui_scale: f32,
    show_conflicts: bool,
    status: String,
    cage_of_cell: Vec<usize>,
    fullscreen: bool,
}

#[derive(Debug, Clone, Copy)]
struct Layout {
    origin_x: i32,
    origin_y: i32,
    cell_px: i32,
    board_px: i32,
}

impl AppState {
    fn new(puzzle: Puzzle, tier: DeductionTier) -> Result<Self, String> {
        let rules = Ruleset::keen_baseline();
        puzzle
            .validate(rules)
            .map_err(|e| format!("puzzle validation failed: {e}"))?;
        let n = puzzle.n as usize;
        let values = vec![0u8; n * n];
        let cage_of_cell = build_cage_index(&puzzle)?;
        Ok(Self {
            puzzle,
            rules,
            tier,
            values,
            selected: Some(0),
            ui_scale: 1.0,
            show_conflicts: true,
            status: "Ready".to_string(),
            cage_of_cell,
            fullscreen: false,
        })
    }

    fn n(&self) -> usize {
        self.puzzle.n as usize
    }

    fn set_status(&mut self, status: impl Into<String>) {
        self.status = status.into();
    }

    fn adjust_scale(&mut self, delta: f32) {
        self.ui_scale = (self.ui_scale + delta).clamp(MIN_SCALE, MAX_SCALE);
        self.set_status(format!("Scale {:.2}x", self.ui_scale));
    }

    fn clear_selected(&mut self) {
        if let Some(idx) = self.selected {
            self.values[idx] = 0;
        }
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let (puzzle, tier) = parse_cli()?;
    let mut state = AppState::new(puzzle, tier)?;

    let sdl = sdl2::init().map_err(|e| e.to_string())?;
    let video = sdl.video().map_err(|e| e.to_string())?;
    let native_mode = video.current_display_mode(0).map_err(|e| e.to_string())?;
    let native_w = native_mode.w.max(1) as u32;
    let native_h = native_mode.h.max(1) as u32;

    let initial_w = ((native_w as f32) * 0.75).clamp(900.0, 1800.0) as u32;
    let initial_h = ((native_h as f32) * 0.75).clamp(700.0, 1200.0) as u32;

    let window = video
        .window("RustyKeen SDL2 demo", initial_w, initial_h)
        .position_centered()
        .resizable()
        .allow_highdpi()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    canvas
        .window_mut()
        .set_minimum_size(640, 480)
        .map_err(|e| e.to_string())?;

    let mut events = sdl.event_pump().map_err(|e| e.to_string())?;
    let mut running = true;

    while running {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => running = false,
                Event::Window {
                    win_event: WindowEvent::SizeChanged(w, h),
                    ..
                } => {
                    state.set_status(format!("Window resized to {}x{}", w, h));
                }
                Event::MouseWheel { y, direction, .. } => {
                    let signed = if direction == MouseWheelDirection::Flipped {
                        -y
                    } else {
                        y
                    };
                    if signed != 0 {
                        state.adjust_scale((signed as f32) * 0.05);
                    }
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    let (out_w, out_h) = canvas.output_size().map_err(|e| e.to_string())?;
                    let layout = compute_layout(out_w, out_h, state.n(), state.ui_scale);
                    if let Some(idx) = board_index_from_pixel(layout, state.n(), x, y) {
                        state.selected = Some(idx);
                        if mouse_btn == MouseButton::Right {
                            state.values[idx] = 0;
                        }
                    }
                }
                Event::KeyDown {
                    keycode: Some(key),
                    repeat: false,
                    ..
                } => {
                    handle_key(&mut canvas, &mut state, key)?;
                }
                _ => {}
            }
        }

        render(&mut canvas, &state)?;
        update_window_title(&mut canvas, &state, native_w, native_h)?;
    }

    Ok(())
}

fn handle_key(
    canvas: &mut Canvas<Window>,
    state: &mut AppState,
    key: Keycode,
) -> Result<(), String> {
    match key {
        Keycode::Escape | Keycode::Q => {
            std::process::exit(0);
        }
        Keycode::F11 => {
            state.fullscreen = !state.fullscreen;
            let mode = if state.fullscreen {
                FullscreenType::Desktop
            } else {
                FullscreenType::Off
            };
            canvas
                .window_mut()
                .set_fullscreen(mode)
                .map_err(|e| e.to_string())?;
            state.set_status(if state.fullscreen {
                "Fullscreen enabled"
            } else {
                "Fullscreen disabled"
            });
        }
        Keycode::Equals | Keycode::Plus | Keycode::KpPlus => state.adjust_scale(0.1),
        Keycode::Minus | Keycode::KpMinus => state.adjust_scale(-0.1),
        Keycode::Left => move_selection(state, 0, -1),
        Keycode::Right => move_selection(state, 0, 1),
        Keycode::Up => move_selection(state, -1, 0),
        Keycode::Down => move_selection(state, 1, 0),
        Keycode::Delete | Keycode::Backspace | Keycode::Kp0 | Keycode::Num0 => {
            state.clear_selected();
        }
        Keycode::R => {
            for value in &mut state.values {
                *value = 0;
            }
            state.set_status("Grid reset");
        }
        Keycode::H => {
            state.show_conflicts = !state.show_conflicts;
            state.set_status(if state.show_conflicts {
                "Conflict highlighting enabled"
            } else {
                "Conflict highlighting disabled"
            });
        }
        Keycode::C => {
            if grid_is_complete_and_valid(state) {
                state.set_status("Current grid is complete and valid");
            } else {
                state.set_status("Current grid is incomplete or invalid");
            }
        }
        Keycode::S => match solve_one_with_deductions(&state.puzzle, state.rules, state.tier) {
            Ok(Some(solution)) => {
                state.values = solution.grid;
                state.set_status("Solved by engine");
            }
            Ok(None) => state.set_status("No solution"),
            Err(e) => state.set_status(format!("Solve error: {e}")),
        },
        Keycode::U => {
            match count_solutions_up_to_with_deductions(&state.puzzle, state.rules, state.tier, 2) {
                Ok(0) => state.set_status("UNSAT"),
                Ok(1) => state.set_status("Unique solution"),
                Ok(_) => state.set_status("Multiple solutions"),
                Err(e) => state.set_status(format!("Count error: {e}")),
            }
        }
        Keycode::F1 => {
            state.tier = DeductionTier::None;
            state.set_status("Tier set: None");
        }
        Keycode::F2 => {
            state.tier = DeductionTier::Easy;
            state.set_status("Tier set: Easy");
        }
        Keycode::F3 => {
            state.tier = DeductionTier::Normal;
            state.set_status("Tier set: Normal");
        }
        Keycode::F4 => {
            state.tier = DeductionTier::Hard;
            state.set_status("Tier set: Hard");
        }
        _ => {
            if let Some(value) = value_from_keycode(key, state.puzzle.n)
                && let Some(idx) = state.selected
            {
                state.values[idx] = value;
            }
        }
    }
    Ok(())
}

fn parse_cli() -> Result<(Puzzle, DeductionTier), String> {
    let args: Vec<String> = std::env::args().collect();
    let mut n = DEFAULT_N;
    let mut desc = DEFAULT_DESC.to_string();
    let mut tier = DeductionTier::Normal;

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--n" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    return Err("missing value for --n".to_string());
                };
                n = v
                    .parse::<u8>()
                    .map_err(|_| format!("invalid --n value: {v}"))?;
            }
            "--desc" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    return Err("missing value for --desc".to_string());
                };
                desc = v.clone();
            }
            "--tier" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    return Err("missing value for --tier".to_string());
                };
                tier = parse_tier(v).ok_or_else(|| format!("invalid tier: {v}"))?;
            }
            "--help" | "-h" => {
                println!("{}", usage());
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}\n\n{}", usage())),
        }
        i += 1;
    }

    let puzzle = parse_keen_desc(n, &desc).map_err(|e| format!("failed to parse puzzle: {e}"))?;
    Ok((puzzle, tier))
}

fn usage() -> &'static str {
    "kenken-sdl2 demo\n\
\n\
USAGE:\n\
  cargo run -p kenken-sdl2 -- [--n <N>] [--desc <DESC>] [--tier <none|easy|normal|hard>]\n\
\n\
CONTROLS:\n\
  Mouse left/right: select / clear cell\n\
  Arrows: move selection\n\
  1-9 / A-F: enter value\n\
  Backspace/Delete/0: clear selected\n\
  S: solve, U: uniqueness up to 2, C: validate current grid, R: reset\n\
  F1..F4: set deduction tier (none/easy/normal/hard)\n\
  +/- or mouse wheel: scale\n\
  F11: toggle fullscreen\n\
  H: toggle conflict highlighting\n\
  Esc/Q: quit\n"
}

fn parse_tier(tier: &str) -> Option<DeductionTier> {
    match tier {
        "none" => Some(DeductionTier::None),
        "easy" => Some(DeductionTier::Easy),
        "normal" => Some(DeductionTier::Normal),
        "hard" => Some(DeductionTier::Hard),
        _ => None,
    }
}

fn build_cage_index(puzzle: &Puzzle) -> Result<Vec<usize>, String> {
    let n = puzzle.n as usize;
    let mut idx = vec![usize::MAX; n * n];
    for (cage_idx, cage) in puzzle.cages.iter().enumerate() {
        for cell in &cage.cells {
            let cell_idx = cell.0 as usize;
            if cell_idx >= idx.len() {
                return Err(format!("cage has out-of-range cell index {}", cell_idx));
            }
            idx[cell_idx] = cage_idx;
        }
    }
    if idx.contains(&usize::MAX) {
        return Err("puzzle has uncovered cells".to_string());
    }
    Ok(idx)
}

fn compute_layout(out_w: u32, out_h: u32, n: usize, ui_scale: f32) -> Layout {
    let min_dim = out_w.min(out_h) as f32;
    let mut cell_px = (min_dim * 0.85 / n as f32) * ui_scale;
    let max_cell_by_w = (out_w as f32 * 0.95) / n as f32;
    let max_cell_by_h = (out_h as f32 * 0.95) / n as f32;
    cell_px = cell_px.min(max_cell_by_w).min(max_cell_by_h).max(20.0);
    let cell_px_i = cell_px.round() as i32;
    let board_px = cell_px_i * n as i32;
    let origin_x = (out_w as i32 - board_px) / 2;
    let origin_y = (out_h as i32 - board_px) / 2;
    Layout {
        origin_x,
        origin_y,
        cell_px: cell_px_i,
        board_px,
    }
}

fn board_index_from_pixel(layout: Layout, n: usize, x: i32, y: i32) -> Option<usize> {
    if x < layout.origin_x
        || y < layout.origin_y
        || x >= layout.origin_x + layout.board_px
        || y >= layout.origin_y + layout.board_px
    {
        return None;
    }
    let col = ((x - layout.origin_x) / layout.cell_px) as usize;
    let row = ((y - layout.origin_y) / layout.cell_px) as usize;
    Some(row * n + col)
}

fn move_selection(state: &mut AppState, d_row: i32, d_col: i32) {
    let n = state.n() as i32;
    let idx = state.selected.unwrap_or(0) as i32;
    let row = idx / n;
    let col = idx % n;
    let next_row = (row + d_row).clamp(0, n - 1);
    let next_col = (col + d_col).clamp(0, n - 1);
    state.selected = Some((next_row * n + next_col) as usize);
}

fn value_from_keycode(key: Keycode, max_n: u8) -> Option<u8> {
    let value = match key {
        Keycode::Num1 | Keycode::Kp1 => 1,
        Keycode::Num2 | Keycode::Kp2 => 2,
        Keycode::Num3 | Keycode::Kp3 => 3,
        Keycode::Num4 | Keycode::Kp4 => 4,
        Keycode::Num5 | Keycode::Kp5 => 5,
        Keycode::Num6 | Keycode::Kp6 => 6,
        Keycode::Num7 | Keycode::Kp7 => 7,
        Keycode::Num8 | Keycode::Kp8 => 8,
        Keycode::Num9 | Keycode::Kp9 => 9,
        Keycode::A => 10,
        Keycode::B => 11,
        Keycode::D => 12,
        Keycode::E => 13,
        Keycode::F => 14,
        _ => return None,
    };
    (value <= max_n).then_some(value)
}

fn update_window_title(
    canvas: &mut Canvas<Window>,
    state: &AppState,
    native_w: u32,
    native_h: u32,
) -> Result<(), String> {
    let (w, h) = canvas.output_size().map_err(|e| e.to_string())?;
    let title = format!(
        "RustyKeen SDL2 demo | native {}x{} window {}x{} | scale {:.2} | tier {:?} | {}",
        native_w, native_h, w, h, state.ui_scale, state.tier, state.status
    );
    canvas
        .window_mut()
        .set_title(&title)
        .map_err(|e| e.to_string())
}

fn render(canvas: &mut Canvas<Window>, state: &AppState) -> Result<(), String> {
    let (out_w, out_h) = canvas.output_size().map_err(|e| e.to_string())?;
    let layout = compute_layout(out_w, out_h, state.n(), state.ui_scale);

    canvas.set_draw_color(Color::RGB(16, 20, 24));
    canvas.clear();

    if let Some(selected) = state.selected {
        let row = selected / state.n();
        let col = selected % state.n();
        let x = layout.origin_x + col as i32 * layout.cell_px;
        let y = layout.origin_y + row as i32 * layout.cell_px;
        canvas.set_draw_color(Color::RGB(40, 72, 108));
        canvas.fill_rect(Rect::new(
            x + 1,
            y + 1,
            (layout.cell_px - 2) as u32,
            (layout.cell_px - 2) as u32,
        ))?;
    }

    draw_grid_lines(canvas, state, layout)?;
    draw_values(canvas, state, layout)?;
    canvas.present();
    Ok(())
}

fn draw_grid_lines(
    canvas: &mut Canvas<Window>,
    state: &AppState,
    layout: Layout,
) -> Result<(), String> {
    let n = state.n() as i32;
    let x0 = layout.origin_x;
    let y0 = layout.origin_y;
    let board_end_x = x0 + layout.board_px;
    let board_end_y = y0 + layout.board_px;

    canvas.set_draw_color(Color::RGB(90, 90, 90));
    for i in 0..=n {
        let x = x0 + i * layout.cell_px;
        canvas.draw_line(Point::new(x, y0), Point::new(x, board_end_y))?;
    }
    for i in 0..=n {
        let y = y0 + i * layout.cell_px;
        canvas.draw_line(Point::new(x0, y), Point::new(board_end_x, y))?;
    }

    for row in 0..state.n() {
        for col in 0..(state.n() - 1) {
            let a = row * state.n() + col;
            let b = row * state.n() + col + 1;
            if state.cage_of_cell[a] != state.cage_of_cell[b] {
                let x = x0 + ((col + 1) as i32 * layout.cell_px) - 1;
                draw_vline_segment(
                    canvas,
                    x,
                    y0 + row as i32 * layout.cell_px,
                    layout.cell_px,
                    3,
                    Color::RGB(220, 220, 220),
                )?;
            }
        }
    }

    for row in 0..(state.n() - 1) {
        for col in 0..state.n() {
            let a = row * state.n() + col;
            let b = (row + 1) * state.n() + col;
            if state.cage_of_cell[a] != state.cage_of_cell[b] {
                let y = y0 + ((row + 1) as i32 * layout.cell_px) - 1;
                draw_hline_segment(
                    canvas,
                    x0 + col as i32 * layout.cell_px,
                    y,
                    layout.cell_px,
                    3,
                    Color::RGB(220, 220, 220),
                )?;
            }
        }
    }

    canvas.set_draw_color(Color::RGB(245, 245, 245));
    draw_hline_segment(
        canvas,
        x0,
        y0,
        layout.board_px,
        4,
        Color::RGB(245, 245, 245),
    )?;
    draw_hline_segment(
        canvas,
        x0,
        board_end_y - 2,
        layout.board_px,
        4,
        Color::RGB(245, 245, 245),
    )?;
    draw_vline_segment(
        canvas,
        x0,
        y0,
        layout.board_px,
        4,
        Color::RGB(245, 245, 245),
    )?;
    draw_vline_segment(
        canvas,
        board_end_x - 2,
        y0,
        layout.board_px,
        4,
        Color::RGB(245, 245, 245),
    )?;
    Ok(())
}

fn draw_hline_segment(
    canvas: &mut Canvas<Window>,
    x: i32,
    y: i32,
    length: i32,
    thickness: i32,
    color: Color,
) -> Result<(), String> {
    canvas.set_draw_color(color);
    canvas.fill_rect(Rect::new(
        x,
        y - (thickness / 2),
        length.max(1) as u32,
        thickness.max(1) as u32,
    ))
}

fn draw_vline_segment(
    canvas: &mut Canvas<Window>,
    x: i32,
    y: i32,
    length: i32,
    thickness: i32,
    color: Color,
) -> Result<(), String> {
    canvas.set_draw_color(color);
    canvas.fill_rect(Rect::new(
        x - (thickness / 2),
        y,
        thickness.max(1) as u32,
        length.max(1) as u32,
    ))
}

fn draw_values(
    canvas: &mut Canvas<Window>,
    state: &AppState,
    layout: Layout,
) -> Result<(), String> {
    for idx in 0..state.values.len() {
        let value = state.values[idx];
        if value == 0 {
            continue;
        }
        let row = idx / state.n();
        let col = idx % state.n();
        let x = layout.origin_x + col as i32 * layout.cell_px;
        let y = layout.origin_y + row as i32 * layout.cell_px;

        let conflict = state.show_conflicts && cell_has_conflict(state, idx);
        let color = if conflict {
            Color::RGB(232, 92, 92)
        } else {
            Color::RGB(245, 245, 245)
        };
        draw_value(canvas, x, y, layout.cell_px, value, color)?;
    }
    Ok(())
}

fn draw_value(
    canvas: &mut Canvas<Window>,
    cell_x: i32,
    cell_y: i32,
    cell_px: i32,
    value: u8,
    color: Color,
) -> Result<(), String> {
    let text = value.to_string();
    let chars: Vec<char> = text.chars().collect();
    let glyph_h = (cell_px as f32 * 0.62) as i32;
    let glyph_w = (glyph_h as f32 * 0.56) as i32;
    let spacing = (glyph_w as f32 * 0.2) as i32;
    let total_w = glyph_w * chars.len() as i32 + spacing * (chars.len().saturating_sub(1) as i32);
    let start_x = cell_x + (cell_px - total_w) / 2;
    let start_y = cell_y + (cell_px - glyph_h) / 2;

    for (i, ch) in chars.iter().enumerate() {
        draw_digit(
            canvas,
            start_x + i as i32 * (glyph_w + spacing),
            start_y,
            glyph_w,
            glyph_h,
            *ch,
            color,
        )?;
    }
    Ok(())
}

fn draw_digit(
    canvas: &mut Canvas<Window>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    digit: char,
    color: Color,
) -> Result<(), String> {
    let Some(segments) = digit_segments(digit) else {
        return Ok(());
    };

    let thickness = (w.max(8) / 6).max(2);
    let half_h = h / 2;
    canvas.set_draw_color(color);

    if segments[0] {
        canvas.fill_rect(Rect::new(x, y, w as u32, thickness as u32))?;
    }
    if segments[1] {
        canvas.fill_rect(Rect::new(
            x + w - thickness,
            y,
            thickness as u32,
            half_h as u32,
        ))?;
    }
    if segments[2] {
        canvas.fill_rect(Rect::new(
            x + w - thickness,
            y + half_h,
            thickness as u32,
            half_h as u32,
        ))?;
    }
    if segments[3] {
        canvas.fill_rect(Rect::new(x, y + h - thickness, w as u32, thickness as u32))?;
    }
    if segments[4] {
        canvas.fill_rect(Rect::new(x, y + half_h, thickness as u32, half_h as u32))?;
    }
    if segments[5] {
        canvas.fill_rect(Rect::new(x, y, thickness as u32, half_h as u32))?;
    }
    if segments[6] {
        canvas.fill_rect(Rect::new(
            x,
            y + half_h - (thickness / 2),
            w as u32,
            thickness as u32,
        ))?;
    }

    Ok(())
}

fn digit_segments(digit: char) -> Option<[bool; 7]> {
    let seg = match digit {
        '0' => [true, true, true, true, true, true, false],
        '1' => [false, true, true, false, false, false, false],
        '2' => [true, true, false, true, true, false, true],
        '3' => [true, true, true, true, false, false, true],
        '4' => [false, true, true, false, false, true, true],
        '5' => [true, false, true, true, false, true, true],
        '6' => [true, false, true, true, true, true, true],
        '7' => [true, true, true, false, false, false, false],
        '8' => [true, true, true, true, true, true, true],
        '9' => [true, true, true, true, false, true, true],
        _ => return None,
    };
    Some(seg)
}

fn cell_has_conflict(state: &AppState, idx: usize) -> bool {
    let value = state.values[idx];
    if value == 0 {
        return false;
    }
    let n = state.n();
    let row = idx / n;
    let col = idx % n;

    for c in 0..n {
        if c != col && state.values[row * n + c] == value {
            return true;
        }
    }
    for r in 0..n {
        if r != row && state.values[r * n + col] == value {
            return true;
        }
    }

    let cage = &state.puzzle.cages[state.cage_of_cell[idx]];
    let cage_values: Vec<u8> = cage
        .cells
        .iter()
        .map(|cell| state.values[cell.0 as usize])
        .collect();
    if cage_values.iter().all(|&v| v != 0) {
        !cage_satisfied(cage, &cage_values)
    } else {
        !cage_partial_consistent(cage, &cage_values)
    }
}

fn grid_is_complete_and_valid(state: &AppState) -> bool {
    state.values.iter().all(|&v| v != 0)
        && (0..state.values.len()).all(|idx| !cell_has_conflict(state, idx))
}

fn cage_partial_consistent(cage: &Cage, values: &[u8]) -> bool {
    match cage.op {
        Op::Eq => values[0] == 0 || values[0] as i32 == cage.target,
        Op::Add => {
            let sum: i32 = values.iter().map(|&v| v as i32).sum();
            sum <= cage.target
        }
        Op::Mul => {
            let product: i32 = values
                .iter()
                .filter(|&&v| v != 0)
                .fold(1i32, |acc, &v| acc.saturating_mul(v as i32));
            product > 0 && cage.target % product == 0
        }
        Op::Sub | Op::Div => true,
    }
}

fn cage_satisfied(cage: &Cage, values: &[u8]) -> bool {
    match cage.op {
        Op::Eq => values.len() == 1 && values[0] as i32 == cage.target,
        Op::Add => values.iter().map(|&v| v as i32).sum::<i32>() == cage.target,
        Op::Mul => {
            values
                .iter()
                .fold(1i32, |acc, &v| acc.saturating_mul(v as i32))
                == cage.target
        }
        Op::Sub => {
            if values.len() != 2 {
                return false;
            }
            (values[0] as i32 - values[1] as i32).abs() == cage.target
        }
        Op::Div => {
            if values.len() != 2 {
                return false;
            }
            let (a, b) = if values[0] >= values[1] {
                (values[0], values[1])
            } else {
                (values[1], values[0])
            };
            b != 0 && (a as i32) == (b as i32).saturating_mul(cage.target)
        }
    }
}
