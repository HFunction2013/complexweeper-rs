// Cross-platform application: window, rendering, input, menus, dialogs.
// Uses macroquad for windowing + 2D rendering, `image` crate for PNG decode.

use crate::assets::Atlas;
use crate::game::*;
use macroquad::color::*;
use macroquad::input::*;
use macroquad::math::{Rect, Vec2};
use macroquad::shapes::*;
use macroquad::text::*;
use macroquad::texture::*;
use macroquad::window::*;
use std::time::{SystemTime, UNIX_EPOCH};

// ------------------------------------------------------------------ constants

pub const APP_TITLE: &str = "Complexweeper";
pub const APP_VERSION: &str = "1.0.0";

const FACE_SIZE: f32 = 24.0;
const FACE_NUDGE: f32 = 2.0;
const CELL_BASE: f32 = 16.0;
const LED_W: f32 = 13.0;
const LED_H: f32 = 23.0;

const C_BTNFACE: Color = Color::new(0.753, 0.753, 0.753, 1.0);
const C_HILIGHT: Color = Color::new(0.875, 0.875, 0.875, 1.0);
const C_SHADOW: Color = Color::new(0.502, 0.502, 0.502, 1.0);
const C_BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
const C_WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
const C_DARKGRAY: Color = Color::new(0.251, 0.251, 0.251, 1.0);
const C_MENU_BG: Color = Color::new(0.878, 0.878, 0.878, 1.0);
const C_MENU_HOVER: Color = Color::new(0.741, 0.769, 0.945, 1.0);
const C_MENU_BORDER: Color = Color::new(0.502, 0.502, 0.502, 1.0);

const MENU_BAR_H: f32 = 22.0;

// ------------------------------------------------------------------ layout

#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub z: f32,
    pub frame: f32,
    pub pad: f32,
    pub header_h: f32,
    pub gap: f32,
    pub box_: f32,
    pub inner_w: f32,
    pub client_w: f32,
    pub client_h: f32,
    pub board_x: f32,
    pub board_y: f32,
    pub cell: f32,
    pub header_x: f32,
    pub header_y: f32,
    pub header_w: f32,
}

fn pow10(n: i32) -> i32 {
    let mut r = 1i32;
    for _ in 0..n {
        r *= 10;
    }
    r
}

fn value_digits(base: i32, v: Option<i32>) -> i32 {
    match v {
        None => base,
        Some(x) => {
            if x >= 0 {
                if x <= pow10(base) - 1 {
                    base
                } else {
                    base + 1
                }
            } else if x >= -(pow10(base - 1) - 1) {
                base
            } else {
                base + 1
            }
        }
    }
}

fn panel_value_digits(imag: bool, v: Option<i32>) -> i32 {
    value_digits(if imag { 3 } else { 4 }, v)
}

fn panel_cells(imag: bool, v: Option<i32>) -> i32 {
    panel_value_digits(imag, v) + if imag { 1 } else { 0 }
}

fn counter_width(z: f32, digits: i32) -> f32 {
    16.0 * z + 2.0 * z + digits as f32 * 13.0 * z + 2.0 * z
}

fn timer_width(z: f32) -> f32 {
    4.0 * 13.0 * z + 2.0 * z
}

fn header_content_width(z: f32) -> f32 {
    let col = counter_width(z, 4);
    4.0 * z + col + 8.0 * z + FACE_SIZE * z + 8.0 * z + timer_width(z) + 4.0 * z
}

// ------------------------------------------------------------------ UI state

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DialogMode {
    None,
    Custom,
    About,
    Help,
    BestScores,
}

#[derive(Debug, Clone)]
struct CustomDialog {
    h: i32,
    w: i32,
    t: [i32; 5], // index 1..4
    err: &'static str,
    focused_field: usize, // 0=h, 1=w, 2..5 = type counts
}

impl Default for CustomDialog {
    fn default() -> Self {
        CustomDialog {
            h: 16,
            w: 30,
            t: [0, 25, 25, 25, 24],
            err: "",
            focused_field: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuOpen {
    None,
    Game,
    Help,
}

// ------------------------------------------------------------------ App

pub struct App {
    pub game: Game,
    atlas: Atlas,
    texture: Texture2D,
    zoom: f32,
    // mouse / interaction state
    face_down: bool,
    face_armed: bool,
    face_flash_until: u64,
    press_cell: i32,
    chord_cell: i32,
    l_down: bool,
    r_down: bool,
    m_down: bool,
    // dialogs
    dialog: DialogMode,
    custom: CustomDialog,
    // menu
    menu_open: MenuOpen,
    menu_hover: i32, // hovered item index in open menu, -1 = none
    // best scores [beginner, intermediate, expert] for each mode
    best_classic: [i32; 3],
    best_complex: [i32; 3],
    // window size
    win_w: f32,
    win_h: f32,
    // for new-highlight
    new_record: bool,
}

impl App {
    pub fn new(atlas: Atlas) -> Self {
        let texture = Texture2D::from_rgba8(
            atlas.width as u16,
            atlas.height as u16,
            &atlas.pixels,
        );
        // Default to Expert preset, Complex mode.
        let mut game = Game::new(GameMode::Complex);
        game.w = PRESETS[2].w;
        game.h = PRESETS[2].h;
        game.mines = PRESETS[2].mines;
        game.new_game(1);

        let mut app = App {
            game,
            atlas,
            texture,
            zoom: 2.0,
            face_down: false,
            face_armed: false,
            face_flash_until: 0,
            press_cell: -1,
            chord_cell: -1,
            l_down: false,
            r_down: false,
            m_down: false,
            dialog: DialogMode::None,
            custom: CustomDialog::default(),
            menu_open: MenuOpen::None,
            menu_hover: -1,
            best_classic: [0; 3],
            best_complex: [0; 3],
            win_w: 800.0,
            win_h: 600.0,
            new_record: false,
        };
        app.load_scores();
        app.resize_window();
        app
    }

    fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Resize the OS window to exactly fit the current layout (menu + header + board).
    fn resize_window(&self) {
        let l = self.layout();
        let mut height = l.client_h;
        #[cfg(target_os = "macos")]
        {
            height += 2.0 * l.pad + l.frame;
        }
        request_new_screen_size(l.client_w, height);
    }

    fn timer_seconds(&self) -> i32 {
        if !self.game.started {
            return 0;
        }
        ((self.game.elapsed_ms / 1000).min(9999)) as i32
    }

    // ------------------------------------------------------------------ layout calc

    fn layout(&self) -> Layout {
        let z = self.zoom;
        let frame = 3.0 * z;
        let pad = 6.0 * z;
        let gap = 6.0 * z;
        let box_ = 3.0 * z;
        let cell = CELL_BASE * z;
        let board_w = self.game.w as f32 * cell + 2.0 * box_;
        let hw = header_content_width(z);
        let inner_w = board_w.max(hw);
        let client_w = inner_w + 2.0 * (frame + pad);
        let counters_h = 4.0 * (26.0 * z) + 3.0 * (2.0 * z);
        let header_h = 2.0 * (2.0 * z) + 2.0 * (3.0 * z) + counters_h;
        let client_h = MENU_BAR_H
            + 2.0 * (frame + pad)
            + header_h
            + gap
            + self.game.h as f32 * cell
            + 2.0 * box_;
        Layout {
            z,
            frame,
            pad,
            header_h,
            gap,
            box_,
            inner_w,
            client_w,
            client_h,
            board_x: frame + pad + (inner_w - board_w) / 2.0 + box_,
            board_y: MENU_BAR_H + frame + pad + header_h + gap + box_,
            cell,
            header_x: frame + pad,
            header_y: MENU_BAR_H + frame + pad,
            header_w: inner_w,
        }
    }

    fn counters_x(&self, l: &Layout) -> f32 {
        l.header_x + 4.0 * l.z
    }
    fn counters_y(&self, l: &Layout) -> f32 {
        l.header_y + 2.0 * l.z + 3.0 * l.z
    }
    fn face_left(&self, l: &Layout) -> f32 {
        let size = FACE_SIZE * l.z;
        let counters_end = self.counters_x(l) + self.counters_width(l);
        let timer_start = self.timer_x(l);
        let clearance = 6.0 * l.z;
        let mut fx = l.header_x + l.header_w / 2.0 - size / 2.0;
        if fx < counters_end + clearance {
            fx = counters_end + clearance;
        }
        if fx + size > timer_start - clearance {
            fx = timer_start - clearance - size;
        }
        if fx < l.header_x {
            fx = l.header_x;
        }
        fx - FACE_NUDGE
    }
    fn face_top(&self, l: &Layout) -> f32 {
        l.header_y + (l.header_h - FACE_SIZE * l.z) / 2.0 - FACE_NUDGE
    }
    fn timer_x(&self, l: &Layout) -> f32 {
        l.header_x + l.header_w - 4.0 * l.z - timer_width(l.z)
    }
    fn timer_y(&self, l: &Layout) -> f32 {
        l.header_y + (l.header_h - 26.0 * l.z) / 2.0
    }

    fn counter_value(&self, t: usize) -> i32 {
        let k = self.game.unmarked(t);
        match t {
            2 | 4 => -k,
            _ => k,
        }
    }
    fn counter_shown(&self, t: usize) -> Option<i32> {
        if !self.game.started {
            return None;
        }
        Some(self.counter_value(t))
    }
    fn counter_imag(t: usize) -> bool {
        t == 3 || t == 4
    }
    fn counters_width(&self, l: &Layout) -> f32 {
        let mut widest = 0.0f32;
        for t in 1..5 {
            let c = counter_width(
                l.z,
                panel_cells(Self::counter_imag(t), self.counter_shown(t)),
            );
            if c > widest {
                widest = c;
            }
        }
        widest
    }

    // ------------------------------------------------------------------ sprite helpers

    fn draw_sprite(&self, name: &str, x: f32, y: f32, dw: f32, dh: f32) {
        if let Some(s) = self.atlas.slot(name) {
            draw_texture_ex(
                &self.texture,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(
                        s.x as f32,
                        s.y as f32,
                        s.w as f32,
                        s.h as f32,
                    )),
                    dest_size: Some(Vec2::new(dw, dh)),
                    ..Default::default()
                },
            );
        }
    }

    fn draw_sprite_sq(&self, name: &str, x: f32, y: f32, size: f32) {
        self.draw_sprite(name, x, y, size, size);
    }

    fn flag_sprite(t: u8) -> &'static str {
        match t {
            1 => "flag_1",
            2 => "flag_2",
            3 => "flag_3",
            _ => "flag_4",
        }
    }
    fn mine_sprite(t: u8) -> &'static str {
        match t {
            1 => "mine_1",
            2 => "mine_2",
            3 => "mine_3",
            _ => "mine_4",
        }
    }
    fn boom_sprite(t: u8) -> &'static str {
        match t {
            1 => "boom_1",
            2 => "boom_2",
            3 => "boom_3",
            _ => "boom_4",
        }
    }
    fn wrong_sprite(t: u8) -> &'static str {
        match t {
            1 => "wrong_1",
            2 => "wrong_2",
            3 => "wrong_3",
            _ => "wrong_4",
        }
    }

    fn face_sprite(&self) -> &'static str {
        if self.face_down {
            return "face_down";
        }
        if self.game.over {
            return if self.game.win { "face_win" } else { "face_dead" };
        }
        if self.board_held() {
            return "face_scan";
        }
        if self.now_ms() < self.face_flash_until {
            return "face_scan";
        }
        "face_normal"
    }

    fn board_held(&self) -> bool {
        self.r_down || self.m_down || (self.l_down && !self.face_down)
    }

    fn flash_face(&mut self) {
        self.face_flash_until = self.now_ms() + 200;
    }

    // ------------------------------------------------------------------ LED drawing

    fn draw_led(&self, x: f32, y: f32, v: Option<i32>, digits: i32, z: f32) -> f32 {
        let lw = LED_W * z;
        let lh = LED_H * z;
        let mut cx = x;
        match v {
            None => {
                for _ in 0..digits {
                    self.draw_sprite("led_blank", cx, y, lw, lh);
                    cx += lw;
                }
                return cx - x;
            }
            Some(val) => {
                if val < 0 {
                    self.draw_sprite("led_minus", cx, y, lw, lh);
                    cx += lw;
                    let mut m = -val;
                    let lim = pow10(digits - 1) - 1;
                    if m > lim {
                        m = lim;
                    }
                    let s = format!("{:0width$}", m, width = (digits - 1) as usize);
                    for ch in s.chars() {
                        self.draw_sprite(&format!("led_{}", ch), cx, y, lw, lh);
                        cx += lw;
                    }
                } else {
                    let mut m = val;
                    let lim = pow10(digits) - 1;
                    if m > lim {
                        m = lim;
                    }
                    let s = format!("{:0width$}", m, width = digits as usize);
                    for ch in s.chars() {
                        self.draw_sprite(&format!("led_{}", ch), cx, y, lw, lh);
                        cx += lw;
                    }
                }
            }
        }
        cx - x
    }

    // ------------------------------------------------------------------ cell sprite

    fn chordable(&self, c: usize) -> bool {
        if c >= self.game.n {
            return false;
        }
        if self.game.over || self.game.open[c] == 0 || self.game.mine[c] != 0 {
            return false;
        }
        let mut buf = [0usize; 8];
        let k = self.game.neighbors(c, &mut buf);
        for &j in &buf[..k] {
            if self.game.open[j] == 0 && self.game.flag[j] == 0 {
                return true;
            }
        }
        false
    }

    fn chord_target(&self, i: usize) -> bool {
        if self.chord_cell < 0 {
            return false;
        }
        let c = self.chord_cell as usize;
        if !self.chordable(c) {
            return false;
        }
        if self.game.open[i] != 0 || self.game.flag[i] != 0 {
            return false;
        }
        let mut buf = [0usize; 8];
        let k = self.game.neighbors(c, &mut buf);
        for &j in &buf[..k] {
            if j == i {
                return true;
            }
        }
        false
    }

    /// Classic mode clue digit -> atlas slot name.
    /// Atlas `num_X` slots are keyed by D=|S|^2 (Complex mode); perfect-square D
    /// values contain plain Arabic numerals — e.g. num_9 draws "3", num_16 draws "4".
    fn classic_clue_sprite(d: i16) -> &'static str {
        match d {
            0 => "num_0",
            1 => "num_1",
            2 => "num_4",
            3 => "num_9",
            4 => "num_16",
            5 => "num_25",
            6 => "num_36",
            7 => "num_49",
            8 => "num_64",
            _ => "blank",
        }
    }

    /// Classic minesweeper number colors (1-8) — kept as fallback only.
    fn classic_number_color(n: i16) -> Color {
        match n {
            1 => Color::new(0.0, 0.0, 1.0, 1.0),       // blue
            2 => Color::new(0.0, 0.502, 0.0, 1.0),     // green
            3 => Color::new(1.0, 0.0, 0.0, 1.0),       // red
            4 => Color::new(0.0, 0.0, 0.502, 1.0),     // dark blue
            5 => Color::new(0.502, 0.0, 0.0, 1.0),     // dark red
            6 => Color::new(0.0, 0.502, 0.502, 1.0),   // teal
            7 => Color::new(0.0, 0.0, 0.0, 1.0),       // black
            8 => Color::new(0.502, 0.502, 0.502, 1.0), // gray
            _ => C_BLACK,
        }
    }

    /// Draw a single cell at (x, y) with the given size. Handles both sprite-based
    /// (Complex mode) and text-based (Classic mode clues) rendering.
    fn draw_cell(&self, i: usize, x: f32, y: f32, size: f32) {
        // Press preview / chord preview
        if self.press_cell >= 0 && self.press_cell as usize == i && self.game.open[i] == 0 {
            self.draw_sprite_sq("blank", x, y, size);
            return;
        }
        if self.chord_target(i) {
            self.draw_sprite_sq("blank", x, y, size);
            return;
        }
        let revealed = self.game.over && !self.game.win;

        if self.game.open[i] != 0 {
            if self.game.mine[i] != 0 {
                let sprite = if self.game.boom == i as i32 {
                    Self::boom_sprite(self.game.mine[i])
                } else {
                    Self::mine_sprite(self.game.mine[i])
                };
                self.draw_sprite_sq(sprite, x, y, size);
                return;
            }
            let d = self.game.clue[i];
            if d == 0 && self.game.neighbor_mine_count(i) == 0 {
                self.draw_sprite_sq("blank", x, y, size);
                return;
            }
            if self.game.mode == GameMode::Classic {
                // Classic: all digits 0-8 map to atlas sprites via perfect-square D values.
                self.draw_sprite_sq(Self::classic_clue_sprite(d), x, y, size);
                return;
            }
            // Complex mode: use number sprite from atlas.
            if let Some(name) = self.atlas.num_sprite(d) {
                self.draw_sprite_sq(name, x, y, size);
            } else {
                self.draw_sprite_sq("blank", x, y, size);
            }
            return;
        }
        if self.game.flag[i] != 0 {
            let right = self.game.mine[i] == self.game.flag[i];
            let sprite = if revealed && !right {
                Self::wrong_sprite(self.game.flag[i])
            } else {
                Self::flag_sprite(self.game.flag[i])
            };
            self.draw_sprite_sq(sprite, x, y, size);
            return;
        }
        if revealed && self.game.mine[i] != 0 {
            self.draw_sprite_sq(Self::mine_sprite(self.game.mine[i]), x, y, size);
            return;
        }
        self.draw_sprite_sq("closed", x, y, size);
    }

    // ------------------------------------------------------------------ 3d border helpers

    fn draw_3d(&self, x: f32, y: f32, w: f32, h: f32, t: f32, raised: bool) {
        let (a, b) = if raised {
            (C_HILIGHT, C_SHADOW)
        } else {
            (C_SHADOW, C_HILIGHT)
        };
        draw_rectangle(x, y, w, t, a); // top
        draw_rectangle(x, y, t, h, a); // left
        draw_rectangle(x, y + h - t, w, t, b); // bottom
        draw_rectangle(x + w - t, y, t, h, b); // right
    }

    // ------------------------------------------------------------------ painting

    fn paint(&self, l: &Layout) {
        // Background
        draw_rectangle(0.0, 0.0, l.client_w, l.client_h, C_BTNFACE);
        // Outer frame (raised)
        self.draw_3d(0.0, 0.0, l.client_w, l.client_h, l.frame, true);

        // Header panel (sunken)
        let hx = l.header_x;
        let hy = l.header_y;
        let hh = l.header_h;
        let hw2 = l.header_w;
        self.draw_3d(hx, hy, hw2, hh, 2.0 * l.z, false);

        // Four counters (Complex mode) or one counter (Classic mode)
        let col_x = self.counters_x(l);
        let mut cy = self.counters_y(l);

        if self.game.mode == GameMode::Complex {
            for t in 1..5 {
                let val = self.counter_shown(t);
                let imag = Self::counter_imag(t);
                let cw2 = counter_width(l.z, panel_cells(imag, val));
                self.draw_3d(col_x, cy, cw2, 26.0 * l.z, 1.0 * l.z, false);
                self.draw_sprite_sq(
                    Self::flag_sprite(t as u8),
                    col_x + l.z + l.z,
                    cy + (26.0 * l.z - 16.0 * l.z) / 2.0,
                    16.0 * l.z,
                );
                let led_x = col_x + l.z + 2.0 * l.z + 16.0 * l.z;
                let led_y = cy + (26.0 * l.z - 23.0 * l.z) / 2.0;
                let vw = self.draw_led(led_x, led_y, val, panel_value_digits(imag, val), l.z);
                if imag {
                    let sprite = if val.is_none() { "led_blank" } else { "led_i" };
                    self.draw_sprite(sprite, led_x + vw, led_y, 13.0 * l.z, 23.0 * l.z);
                }
                cy += 26.0 * l.z + 2.0 * l.z;
            }
        } else {
            // Classic: single mine counter
            let val: Option<i32> = if self.game.started {
                Some(self.game.mines as i32 - self.game.flags_total() as i32)
            } else {
                None
            };
            let cw2 = counter_width(l.z, panel_cells(false, val));
            self.draw_3d(col_x, cy, cw2, 26.0 * l.z, 1.0 * l.z, false);
            self.draw_sprite_sq(
                "flag_1",
                col_x + l.z + l.z,
                cy + (26.0 * l.z - 16.0 * l.z) / 2.0,
                16.0 * l.z,
            );
            let led_x = col_x + l.z + 2.0 * l.z + 16.0 * l.z;
            let led_y = cy + (26.0 * l.z - 23.0 * l.z) / 2.0;
            self.draw_led(led_x, led_y, val, panel_value_digits(false, val), l.z);
        }

        // Timer
        let tx = self.timer_x(l);
        let ty = self.timer_y(l);
        let tv: Option<i32> = if self.game.started {
            Some(self.timer_seconds())
        } else {
            None
        };
        self.draw_3d(tx, ty, timer_width(l.z), 26.0 * l.z, 1.0 * l.z, false);
        self.draw_led(
            tx + l.z,
            ty + (26.0 * l.z - 23.0 * l.z) / 2.0,
            tv,
            panel_value_digits(false, tv),
            l.z,
        );

        // Face button
        self.draw_sprite_sq(
            self.face_sprite(),
            self.face_left(l),
            self.face_top(l),
            FACE_SIZE * l.z,
        );

        // Board sunken frame
        let bx = l.board_x - l.box_;
        let by = l.board_y - l.box_;
        let bw = self.game.w as f32 * l.cell + 2.0 * l.box_;
        let bh = self.game.h as f32 * l.cell + 2.0 * l.box_;
        self.draw_3d(bx, by, bw, bh, l.box_, false);

        // Cells
        for i in 0..self.game.n {
            let r = (i / self.game.w as usize) as f32;
            let c = (i % self.game.w as usize) as f32;
            let x = l.board_x + c * l.cell;
            let y = l.board_y + r * l.cell;
            self.draw_cell(i, x, y, l.cell);
        }
    }

    // ------------------------------------------------------------------ hit testing

    fn cell_at(&self, l: &Layout, px: f32, py: f32) -> i32 {
        let cx = px - l.board_x;
        let cy = py - l.board_y;
        if cx < 0.0 || cy < 0.0 {
            return -1;
        }
        let c = (cx / l.cell) as i32;
        let r = (cy / l.cell) as i32;
        if c < 0 || r < 0 || c >= self.game.w as i32 || r >= self.game.h as i32 {
            return -1;
        }
        r * self.game.w as i32 + c
    }

    fn in_face(&self, l: &Layout, px: f32, py: f32) -> bool {
        let fw = FACE_SIZE * l.z;
        let fx = self.face_left(l);
        let fy = self.face_top(l);
        px >= fx && py >= fy && px < fx + fw && py < fy + fw
    }

    // ------------------------------------------------------------------ game actions

    fn start_new_game(&mut self, seed_override: Option<u32>) {
        let seed = seed_override.unwrap_or_else(|| self.now_ms() as u32);
        self.game.new_game(seed);
        self.press_cell = -1;
        self.chord_cell = -1;
        self.l_down = false;
        self.r_down = false;
        self.m_down = false;
        self.face_down = false;
        self.face_armed = false;
        self.face_flash_until = 0;
        self.new_record = false;
        self.resize_window();
    }

    fn set_preset(&mut self, idx: i32) {
        if idx >= 0 && idx < 3 {
            self.game.w = PRESETS[idx as usize].w;
            self.game.h = PRESETS[idx as usize].h;
            self.game.mines = PRESETS[idx as usize].mines;
            self.game.type_count = [0; 5];
        }
        self.start_new_game(None);
    }

    fn do_expand(&mut self, c: usize) {
        let m0 = self.game.moves;
        self.game.try_expand(c);
        self.after_game_action();
        if self.game.moves != m0 && !self.game.over {
            self.flash_face();
        }
    }

    fn after_game_action(&mut self) {
        if !self.game.over {
            return;
        }
        if self.game.t0 != 0 {
            self.game.elapsed_ms = (self.now_ms() - self.game.t0 as u64) as u32;
        }
        if !self.game.win {
            return;
        }
        let idx = self.preset_index();
        if idx < 0 {
            return;
        }
        let sec = (self.game.elapsed_ms / 1000).max(1) as i32;
        let u = idx as usize;
        let scores = if self.game.mode == GameMode::Classic {
            &mut self.best_classic
        } else {
            &mut self.best_complex
        };
        if scores[u] != 0 && sec >= scores[u] {
            return;
        }
        scores[u] = sec;
        self.save_scores();
        self.new_record = true;
        self.dialog = DialogMode::BestScores;
    }

    fn preset_index(&self) -> i32 {
        for (i, p) in PRESETS.iter().enumerate() {
            if self.game.w == p.w && self.game.h == p.h && self.game.mines == p.mines {
                return i as i32;
            }
        }
        -1
    }

    // ------------------------------------------------------------------ scores persistence

    fn scores_path() -> Option<std::path::PathBuf> {
        let dirs = directories::ProjectDirs::from("org", "complexweeper", "Complexweeper")?;
        Some(dirs.data_dir().join("scores.json"))
    }

    fn load_scores(&mut self) {
        if let Some(path) = Self::scores_path() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(c) = v.get("classic").and_then(|x| x.as_array()) {
                        for (i, val) in c.iter().enumerate().take(3) {
                            self.best_classic[i] = val.as_i64().unwrap_or(0) as i32;
                        }
                    }
                    if let Some(c) = v.get("complex").and_then(|x| x.as_array()) {
                        for (i, val) in c.iter().enumerate().take(3) {
                            self.best_complex[i] = val.as_i64().unwrap_or(0) as i32;
                        }
                    }
                }
            }
        }
    }

    fn save_scores(&self) {
        if let Some(path) = Self::scores_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let v = serde_json::json!({
                "classic": self.best_classic,
                "complex": self.best_complex,
            });
            let _ = std::fs::write(&path, serde_json::to_string_pretty(&v).unwrap());
        }
    }

    // ------------------------------------------------------------------ menu

    fn game_menu_items(&self) -> Vec<(&'static str, i32, bool)> {
        // (label, command id, separator_after)
        vec![
            ("New Game\tF2", 100, true),
            ("Beginner\t9x9 - 10", 101, false),
            ("Intermediate\t16x16 - 40", 102, false),
            ("Expert\t30x16 - 99", 103, false),
            ("Custom...", 104, true),
            ("Mode: Complex", 150, true),
            ("Best Scores...", 106, true),
            ("Zoom 100%", 110, false),
            ("Zoom 200%", 111, false),
            ("Zoom 300%", 112, true),
            ("Exit", 105, false),
        ]
    }

    fn help_menu_items() -> Vec<(&'static str, i32, bool)> {
        vec![
            ("How to Play", 200, true),
            ("About Complexweeper", 202, false),
        ]
    }

    fn draw_menu_bar(&self) {
        draw_rectangle(0.0, 0.0, self.win_w, MENU_BAR_H, C_MENU_BG);
        draw_line(0.0, MENU_BAR_H, self.win_w, MENU_BAR_H, 1.0, C_SHADOW);

        let game_label = "Game";
        let help_label = "Help";
        let pad = 12.0;
        let game_w = measure_text(game_label, None, 14, 1.0).width + pad * 2.0;
        let help_x = game_w;
        let help_w = measure_text(help_label, None, 14, 1.0).width + pad * 2.0;

        let (mx, my) = mouse_position();
        let game_hover = mx < game_w && my < MENU_BAR_H;
        let help_hover = mx >= help_x && mx < help_x + help_w && my < MENU_BAR_H;

        if self.menu_open == MenuOpen::Game || game_hover {
            draw_rectangle(0.0, 0.0, game_w, MENU_BAR_H, C_MENU_HOVER);
        }
        if self.menu_open == MenuOpen::Help || help_hover {
            draw_rectangle(help_x, 0.0, help_w, MENU_BAR_H, C_MENU_HOVER);
        }

        draw_text(game_label, pad, 16.0, 14.0, C_BLACK);
        draw_text(help_label, help_x + pad, 16.0, 14.0, C_BLACK);
    }

    fn draw_dropdown(&self, x: f32, items: &[(&str, i32, bool)], hover: i32) {
        let item_h = 22.0;
        let mut max_w = 180.0f32;
        for (label, _, _) in items {
            let w = measure_text(label, None, 14, 1.0).width + 24.0;
            if w > max_w {
                max_w = w;
            }
        }
        let total_h = items.len() as f32 * item_h;
        draw_rectangle(x, MENU_BAR_H, max_w, total_h, C_MENU_BG);
        draw_rectangle_lines(x, MENU_BAR_H, max_w, total_h, 1.0, C_MENU_BORDER);

        let mut y = MENU_BAR_H;
        for (i, (label, _, sep)) in items.iter().enumerate() {
            if hover == i as i32 {
                draw_rectangle(x + 1.0, y + 1.0, max_w - 2.0, item_h - 2.0, C_MENU_HOVER);
            }
            draw_text(label, x + 12.0, y + 16.0, 14.0, C_BLACK);
            if *sep {
                draw_line(
                    x + 4.0,
                    y + item_h - 1.0,
                    x + max_w - 4.0,
                    y + item_h - 1.0,
                    1.0,
                    C_SHADOW,
                );
            }
            y += item_h;
        }
    }

    fn menu_item_at(&self, x: f32, items: &[(&str, i32, bool)], mx: f32, my: f32) -> i32 {
        let item_h = 22.0;
        if mx < x || my < MENU_BAR_H {
            return -1;
        }
        let idx = ((my - MENU_BAR_H) / item_h) as i32;
        if idx < 0 || idx >= items.len() as i32 {
            return -1;
        }
        idx
    }

    fn handle_menu_command(&mut self, id: i32) {
        match id {
            100 => self.start_new_game(None),
            101 => self.set_preset(0),
            102 => self.set_preset(1),
            103 => self.set_preset(2),
            104 => {
                self.custom.h = self.game.h as i32;
                self.custom.w = self.game.w as i32;
                let mut tc = self.game.type_count;
                let any: u16 = tc[1..5].iter().sum();
                if any == 0 {
                    tc = split_evenly(self.game.mines);
                }
                for t in 1..5 {
                    self.custom.t[t] = tc[t] as i32;
                }
                self.custom.err = "";
                self.custom.focused_field = 0;
                self.dialog = DialogMode::Custom;
            }
            105 => {
                // Exit — macroquad doesn't have a direct quit, but we can request.
                std::process::exit(0);
            }
            106 => self.dialog = DialogMode::BestScores,
            110 => {
                self.zoom = 1.0;
                self.resize_window();
            }
            111 => {
                self.zoom = 2.0;
                self.resize_window();
            }
            112 => {
                self.zoom = 3.0;
                self.resize_window();
            }
            150 => {
                // Toggle mode
                self.game.mode = if self.game.mode == GameMode::Classic {
                    GameMode::Complex
                } else {
                    GameMode::Classic
                };
                // Reset type_count for classic mode
                if self.game.mode == GameMode::Classic {
                    self.game.type_count = [0; 5];
                }
                self.start_new_game(None);
            }
            200 => self.dialog = DialogMode::Help,
            202 => self.dialog = DialogMode::About,
            _ => {}
        }
    }

    // ------------------------------------------------------------------ custom dialog

    fn draw_custom_dialog(&self) {
        let dw = 340.0;
        let dh = 280.0;
        let dx = (self.win_w - dw) / 2.0;
        let dy = (self.win_h - dh) / 2.0;

        // Dim background
        draw_rectangle(0.0, 0.0, self.win_w, self.win_h, Color::new(0.0, 0.0, 0.0, 0.3));
        // Dialog box
        draw_rectangle(dx, dy, dw, dh, C_WHITE);
        draw_rectangle_lines(dx, dy, dw, dh, 2.0, C_SHADOW);
        // Title bar
        draw_rectangle(dx, dy, dw, 28.0, Color::new(0.0, 0.2, 0.6, 1.0));
        draw_text("Custom Field", dx + 10.0, dy + 20.0, 14.0, C_WHITE);

        let fields = [
            ("Height (H):", 0, "9 - 30 rows"),
            ("Width (W):", 1, "9 - 40 cols"),
        ];
        let mut y = dy + 44.0;
        for (label, idx, hint) in &fields {
            draw_text(label, dx + 14.0, y + 16.0, 14.0, C_BLACK);
            let fx = dx + 100.0;
            let focused = self.custom.focused_field == *idx;
            draw_rectangle(fx, y, 70.0, 22.0, if focused { Color::new(0.9, 0.95, 1.0, 1.0) } else { C_WHITE });
            draw_rectangle_lines(fx, y, 70.0, 22.0, 1.0, if focused { Color::new(0.0, 0.3, 0.8, 1.0) } else { C_SHADOW });
            let val = if *idx == 0 { self.custom.h } else { self.custom.w };
            draw_text(&val.to_string(), fx + 6.0, y + 16.0, 14.0, C_BLACK);
            draw_text(hint, dx + 180.0, y + 16.0, 12.0, C_DARKGRAY);
            y += 30.0;
        }

        y += 4.0;
        let type_labels = ["+Real:", "-Real:", "+Imag:", "-Imag:"];
        for k in 0..4 {
            let col = if k % 2 == 0 { 0.0 } else { 160.0 };
            let row = (k / 2) as f32;
            let yy = y + row * 30.0;
            let idx = 2 + k;
            draw_text(type_labels[k], dx + 14.0 + col, yy + 16.0, 14.0, C_BLACK);
            let fx = dx + 82.0 + col;
            let focused = self.custom.focused_field == idx;
            draw_rectangle(fx, yy, 62.0, 22.0, if focused { Color::new(0.9, 0.95, 1.0, 1.0) } else { C_WHITE });
            draw_rectangle_lines(fx, yy, 62.0, 22.0, 1.0, if focused { Color::new(0.0, 0.3, 0.8, 1.0) } else { C_SHADOW });
            draw_text(&self.custom.t[k + 1].to_string(), fx + 6.0, yy + 16.0, 14.0, C_BLACK);
        }
        y += 66.0;

        // Split button
        let split_x = dx + 14.0;
        draw_rectangle(split_x, y, 110.0, 24.0, C_MENU_BG);
        draw_rectangle_lines(split_x, y, 110.0, 24.0, 1.0, C_SHADOW);
        draw_text("Split Evenly", split_x + 10.0, y + 17.0, 13.0, C_BLACK);

        // Error text
        if !self.custom.err.is_empty() {
            draw_text(self.custom.err, dx + 134.0, y + 17.0, 12.0, Color::new(0.8, 0.0, 0.0, 1.0));
        }
        y += 34.0;

        // OK / Cancel
        let ok_x = dx + 122.0;
        let cancel_x = dx + 218.0;
        draw_rectangle(ok_x, y, 88.0, 26.0, C_MENU_BG);
        draw_rectangle_lines(ok_x, y, 88.0, 26.0, 1.0, C_SHADOW);
        draw_text("OK", ok_x + 34.0, y + 18.0, 14.0, C_BLACK);
        draw_rectangle(cancel_x, y, 88.0, 26.0, C_MENU_BG);
        draw_rectangle_lines(cancel_x, y, 88.0, 26.0, 1.0, C_SHADOW);
        draw_text("Cancel", cancel_x + 24.0, y + 18.0, 14.0, C_BLACK);
    }

    fn custom_dialog_hit(&self, mx: f32, my: f32) -> &'static str {
        let dw = 340.0;
        let dh = 280.0;
        let dx = (self.win_w - dw) / 2.0;
        let dy = (self.win_h - dh) / 2.0;

        // Field rects
        let fields_y = dy + 44.0;
        // h field
        if mx >= dx + 100.0 && mx <= dx + 170.0 && my >= fields_y && my <= fields_y + 22.0 {
            return "field_h";
        }
        // w field
        if mx >= dx + 100.0 && mx <= dx + 170.0 && my >= fields_y + 30.0 && my <= fields_y + 52.0 {
            return "field_w";
        }
        // type fields
        let type_y = fields_y + 64.0;
        for k in 0..4 {
            let col = if k % 2 == 0 { 0.0 } else { 160.0 };
            let row = (k / 2) as f32;
            let yy = type_y + row * 30.0;
            let fx = dx + 82.0 + col;
            if mx >= fx && mx <= fx + 62.0 && my >= yy && my <= yy + 22.0 {
                return match k {
                    0 => "field_t1",
                    1 => "field_t2",
                    2 => "field_t3",
                    _ => "field_t4",
                };
            }
        }
        // buttons
        let btn_y = type_y + 66.0;
        if mx >= dx + 14.0 && mx <= dx + 124.0 && my >= btn_y && my <= btn_y + 24.0 {
            return "split";
        }
        let ok_y = btn_y + 34.0;
        if mx >= dx + 122.0 && mx <= dx + 210.0 && my >= ok_y && my <= ok_y + 26.0 {
            return "ok";
        }
        if mx >= dx + 218.0 && mx <= dx + 306.0 && my >= ok_y && my <= ok_y + 26.0 {
            return "cancel";
        }
        "none"
    }

    fn apply_custom(&mut self) -> bool {
        let h = self.custom.h;
        let w = self.custom.w;
        let mut sum = 0i32;
        for t in 1..5 {
            sum += self.custom.t[t];
        }
        if h < 9 || h > 30 {
            self.custom.err = "Height must be 9 - 30.";
            return false;
        }
        if w < 9 || w > 40 {
            self.custom.err = "Width must be 9 - 40.";
            return false;
        }
        if sum < 1 {
            self.custom.err = "At least 1 mine total.";
            return false;
        }
        let maxm = w * h - 9;
        if sum > maxm {
            self.custom.err = "Total exceeds limit (cells - 9).";
            return false;
        }
        self.custom.err = "";
        self.game.w = w as u16;
        self.game.h = h as u16;
        self.game.mines = sum as u16;
        if self.game.mode == GameMode::Complex {
            self.game.type_count = [
                0,
                self.custom.t[1] as u16,
                self.custom.t[2] as u16,
                self.custom.t[3] as u16,
                self.custom.t[4] as u16,
            ];
        } else {
            self.game.type_count = [0; 5];
        }
        true
    }

    // ------------------------------------------------------------------ message boxes

    fn draw_message_box(&self, title: &str, lines: &[&str]) {
        let padding = 20.0;
        let line_h = 20.0;
        let mut max_w = 200.0f32;
        for l in lines {
            let w = measure_text(l, None, 14, 1.0).width + padding * 2.0;
            if w > max_w {
                max_w = w;
            }
        }
        let dh = 60.0 + lines.len() as f32 * line_h + 40.0;
        let dw = max_w;
        let dx = (self.win_w - dw) / 2.0;
        let dy = (self.win_h - dh) / 2.0;

        draw_rectangle(0.0, 0.0, self.win_w, self.win_h, Color::new(0.0, 0.0, 0.0, 0.3));
        draw_rectangle(dx, dy, dw, dh, C_WHITE);
        draw_rectangle_lines(dx, dy, dw, dh, 2.0, C_SHADOW);
        draw_rectangle(dx, dy, dw, 28.0, Color::new(0.0, 0.2, 0.6, 1.0));
        draw_text(title, dx + 10.0, dy + 20.0, 14.0, C_WHITE);

        let mut y = dy + 48.0;
        for l in lines {
            draw_text(l, dx + padding, y, 14.0, C_BLACK);
            y += line_h;
        }

        // OK button
        let ok_w = 80.0;
        let ok_x = dx + (dw - ok_w) / 2.0;
        let ok_y = dy + dh - 36.0;
        draw_rectangle(ok_x, ok_y, ok_w, 26.0, C_MENU_BG);
        draw_rectangle_lines(ok_x, ok_y, ok_w, 26.0, 1.0, C_SHADOW);
        draw_text("OK", ok_x + 30.0, ok_y + 18.0, 14.0, C_BLACK);
    }

    fn message_box_hit(&self, mx: f32, my: f32, lines: &[&str]) -> bool {
        let padding = 20.0;
        let line_h = 20.0;
        let mut max_w = 200.0f32;
        for l in lines {
            let w = measure_text(l, None, 14, 1.0).width + padding * 2.0;
            if w > max_w {
                max_w = w;
            }
        }
        let dh = 60.0 + lines.len() as f32 * line_h + 40.0;
        let dw = max_w;
        let dx = (self.win_w - dw) / 2.0;
        let dy = (self.win_h - dh) / 2.0;
        let ok_w = 80.0;
        let ok_x = dx + (dw - ok_w) / 2.0;
        let ok_y = dy + dh - 36.0;
        mx >= ok_x && mx <= ok_x + ok_w && my >= ok_y && my <= ok_y + 26.0
    }

    fn about_lines() -> Vec<&'static str> {
        vec![
            "Complexweeper 1.0.0",
            "Based on Microsoft Minesweeper",
            "(original authors: Robert Donner, Curt Johnson)",
            "",
            "Image assets: Microsoft (original minesweeper assets);",
            "Qingyue Xiao (new assets).",
            "Code licensed under GPL-3.0.",
            "",
            "This program is free software.",
            "Not affiliated with Microsoft.",
        ]
    }

    fn help_lines(&self) -> Vec<String> {
        if self.game.mode == GameMode::Classic {
            vec![
                "Left-click to reveal a cell.".to_string(),
                "Right-click to place/remove a flag.".to_string(),
                "Middle-click or both buttons to chord.".to_string(),
                "F2 to start a new game.".to_string(),
                "".to_string(),
                "Numbers show how many mines are adjacent.".to_string(),
                "Reveal all non-mine cells to win.".to_string(),
            ]
        } else {
            vec![
                "Four mine types: +Real, -Real, +Imag, -Imag.".to_string(),
                "Left-click to reveal a cell.".to_string(),
                "Right-click cycles flags: +R, -R, +I, -I.".to_string(),
                "Middle-click or both buttons to chord.".to_string(),
                "F2 to start a new game.".to_string(),
                "".to_string(),
                "Numbers show |sum of adjacent mines|^2.".to_string(),
                "Chord when flag count matches and real/imag".to_string(),
                "ratio matches (or is swapped).".to_string(),
            ]
        }
    }

    fn best_scores_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if self.new_record {
            lines.push("*** New Record! ***".to_string());
            lines.push("".to_string());
        }
        let scores = if self.game.mode == GameMode::Classic {
            &self.best_classic
        } else {
            &self.best_complex
        };
        let mode_name = if self.game.mode == GameMode::Classic {
            "Classic"
        } else {
            "Complex"
        };
        lines.push(format!("Best Times ({})", mode_name));
        lines.push("".to_string());
        let labels = ["Beginner", "Intermediate", "Expert"];
        for (i, label) in labels.iter().enumerate() {
            let s = scores[i];
            let txt = if s > 0 {
                format!("{}      {} sec", label, s)
            } else {
                format!("{}      ---", label)
            };
            lines.push(txt);
        }
        lines
    }

    // ------------------------------------------------------------------ text input for dialog

    fn handle_dialog_text_input(&mut self) {
        if self.dialog != DialogMode::Custom {
            return;
        }
        // Backspace
        if is_key_pressed(KeyCode::Backspace) {
            let idx = self.custom.focused_field;
            let val = match idx {
                0 => &mut self.custom.h,
                1 => &mut self.custom.w,
                2..=5 => &mut self.custom.t[idx],
                _ => return,
            };
            *val /= 10;
            return;
        }
        // Tab cycles focus
        if is_key_pressed(KeyCode::Tab) {
            let max_field = if self.game.mode == GameMode::Classic { 2 } else { 6 };
            self.custom.focused_field = (self.custom.focused_field + 1) % max_field;
            return;
        }
        // Enter = OK
        if is_key_pressed(KeyCode::Enter) {
            if self.apply_custom() {
                self.dialog = DialogMode::None;
                self.start_new_game(None);
            }
            return;
        }
        if is_key_pressed(KeyCode::Escape) {
            self.dialog = DialogMode::None;
            return;
        }
        // Digit input
        while let Some(ch) = get_char_pressed() {
            if ch.is_ascii_digit() {
                let idx = self.custom.focused_field;
                let val = match idx {
                    0 => &mut self.custom.h,
                    1 => &mut self.custom.w,
                    2..=5 => &mut self.custom.t[idx],
                    _ => continue,
                };
                let d = ch.to_digit(10).unwrap() as i32;
                let new_val = *val * 10 + d;
                if new_val <= 9999 {
                    *val = new_val;
                }
            }
        }
    }

    // ------------------------------------------------------------------ main update / draw

    pub fn update(&mut self) {
        self.win_w = screen_width();
        self.win_h = screen_height();

        // Update elapsed time
        if self.game.started && !self.game.over && self.game.t0 != 0 {
            self.game.elapsed_ms = (self.now_ms() - self.game.t0 as u64) as u32;
        }

        // Handle dialog input first (modal)
        if self.dialog != DialogMode::None {
            self.handle_dialog_text_input();
            self.handle_dialog_mouse();
            return;
        }

        // Keyboard shortcuts
        if is_key_pressed(KeyCode::F2) {
            self.start_new_game(None);
        }

        self.handle_menu_mouse();
        if self.menu_open != MenuOpen::None {
            return; // menu is modal for mouse
        }

        self.handle_board_mouse();
    }

    fn handle_dialog_mouse(&mut self) {
        let (mx, my) = mouse_position();
        let clicked = is_mouse_button_pressed(MouseButton::Left);

        match self.dialog {
            DialogMode::Custom => {
                if clicked {
                    match self.custom_dialog_hit(mx, my) {
                        "field_h" => self.custom.focused_field = 0,
                        "field_w" => self.custom.focused_field = 1,
                        "field_t1" => self.custom.focused_field = 2,
                        "field_t2" => self.custom.focused_field = 3,
                        "field_t3" => self.custom.focused_field = 4,
                        "field_t4" => self.custom.focused_field = 5,
                        "split" => {
                            let mut sum = 0i32;
                            for t in 1..5 {
                                sum += self.custom.t[t];
                            }
                            if sum <= 0 {
                                sum = 99;
                            }
                            let sp = split_evenly(sum.min(999) as u16);
                            for t in 1..5 {
                                self.custom.t[t] = sp[t] as i32;
                            }
                        }
                        "ok" => {
                            if self.apply_custom() {
                                self.dialog = DialogMode::None;
                                self.start_new_game(None);
                            }
                        }
                        "cancel" => {
                            self.dialog = DialogMode::None;
                        }
                        _ => {}
                    }
                }
            }
            DialogMode::About => {
                let lines = Self::about_lines();
                let line_refs: Vec<&str> = lines.iter().map(|s| *s).collect();
                if clicked && self.message_box_hit(mx, my, &line_refs) {
                    self.dialog = DialogMode::None;
                }
            }
            DialogMode::Help => {
                let lines = self.help_lines();
                let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
                if clicked && self.message_box_hit(mx, my, &line_refs) {
                    self.dialog = DialogMode::None;
                }
            }
            DialogMode::BestScores => {
                let lines = self.best_scores_lines();
                let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
                if clicked && self.message_box_hit(mx, my, &line_refs) {
                    self.dialog = DialogMode::None;
                    self.new_record = false;
                }
            }
            DialogMode::None => {}
        }
    }

    fn handle_menu_mouse(&mut self) {
        let (mx, my) = mouse_position();
        let clicked = is_mouse_button_pressed(MouseButton::Left);

        let game_label = "Game";
        let help_label = "Help";
        let pad = 12.0;
        let game_w = measure_text(game_label, None, 14, 1.0).width + pad * 2.0;
        let help_x = game_w;
        let help_w = measure_text(help_label, None, 14, 1.0).width + pad * 2.0;

        let on_game_menu = mx < game_w && my < MENU_BAR_H;
        let on_help_menu = mx >= help_x && mx < help_x + help_w && my < MENU_BAR_H;

        if clicked {
            if on_game_menu {
                self.menu_open = if self.menu_open == MenuOpen::Game {
                    MenuOpen::None
                } else {
                    MenuOpen::Game
                };
                self.menu_hover = -1;
                return;
            }
            if on_help_menu {
                self.menu_open = if self.menu_open == MenuOpen::Help {
                    MenuOpen::None
                } else {
                    MenuOpen::Help
                };
                self.menu_hover = -1;
                return;
            }
            // If a menu is open, check dropdown item click
            if self.menu_open != MenuOpen::None {
                let items = match self.menu_open {
                    MenuOpen::Game => self.game_menu_items(),
                    MenuOpen::Help => App::help_menu_items(),
                    _ => vec![],
                };
                let x = if self.menu_open == MenuOpen::Game { 0.0 } else { help_x };
                let idx = self.menu_item_at(x, &items, mx, my);
                if idx >= 0 {
                    let (_, id, _) = items[idx as usize];
                    self.handle_menu_command(id);
                }
                self.menu_open = MenuOpen::None;
                self.menu_hover = -1;
                return;
            }
        } else {
            // Hover tracking
            if self.menu_open != MenuOpen::None {
                let items = match self.menu_open {
                    MenuOpen::Game => self.game_menu_items(),
                    MenuOpen::Help => App::help_menu_items(),
                    _ => vec![],
                };
                let x = if self.menu_open == MenuOpen::Game { 0.0 } else { help_x };
                self.menu_hover = self.menu_item_at(x, &items, mx, my);
            }
        }
    }

    fn handle_board_mouse(&mut self) {
        let l = self.layout();
        let (mx, my) = mouse_position();

        // Left button
        if is_mouse_button_pressed(MouseButton::Left) {
            let on_face = self.in_face(&l, mx, my);
            self.l_down = true;
            self.face_armed = on_face;
            self.face_down = on_face;
            let c = self.cell_at(&l, mx, my);
            if self.r_down {
                self.chord_cell = c;
                self.press_cell = -1;
            } else {
                self.press_cell = if c >= 0
                    && !on_face
                    && !self.game.over
                    && self.game.open[c as usize] == 0
                {
                    c
                } else {
                    -1
                };
            }
        }

        // Mouse move (for press/chord preview)
        if self.l_down || self.r_down || self.m_down {
            let c = self.cell_at(&l, mx, my);
            if self.chord_cell >= 0 {
                let next = if c >= 0 { c } else { -1 };
                if next != self.chord_cell {
                    self.chord_cell = next;
                }
            } else if self.press_cell >= 0 {
                let next = if c >= 0 && self.game.open[c as usize] == 0 {
                    c
                } else {
                    -1
                };
                if next != self.press_cell {
                    self.press_cell = next;
                }
            }
        }

        // Left release
        if is_mouse_button_released(MouseButton::Left) {
            let held = self.press_cell;
            let chord = self.chord_cell;
            let was_face = self.face_armed;
            self.l_down = false;
            self.press_cell = -1;
            self.chord_cell = -1;
            self.face_down = false;
            self.face_armed = false;

            if was_face && self.in_face(&l, mx, my) {
                self.start_new_game(None);
                return;
            }
            if chord >= 0 {
                if self.cell_at(&l, mx, my) == chord {
                    self.do_expand(chord as usize);
                }
                return;
            }
            if held >= 0
                && self.cell_at(&l, mx, my) == held
                && !self.game.over
                && self.game.flag[held as usize] == 0
            {
                let c = held as usize;
                let covered = self.game.open[c] == 0;
                if !self.game.started {
                    self.game.start_at(c, self.now_ms() as u32);
                    self.game.set_msg(Msg::Started);
                } else {
                    self.game.reveal(c, self.now_ms() as u32);
                    self.after_game_action();
                }
                if covered && !self.game.over {
                    self.flash_face();
                }
            }
        }

        // Right button
        if is_mouse_button_pressed(MouseButton::Right) {
            self.r_down = true;
            let c = self.cell_at(&l, mx, my);
            if self.l_down {
                self.chord_cell = c;
                self.press_cell = -1;
            } else if c >= 0 && !self.game.over && self.game.open[c as usize] == 0 {
                if self.game.cycle_flag(c as usize) {
                    self.flash_face();
                }
            }
        }
        if is_mouse_button_released(MouseButton::Right) {
            self.r_down = false;
            let chord = self.chord_cell;
            if chord >= 0 {
                self.chord_cell = -1;
                self.press_cell = -1;
                self.face_down = false;
                if self.cell_at(&l, mx, my) == chord {
                    self.do_expand(chord as usize);
                }
            }
        }

        // Middle button
        if is_mouse_button_pressed(MouseButton::Middle) {
            self.m_down = true;
            let c = self.cell_at(&l, mx, my);
            self.chord_cell = c;
            self.press_cell = -1;
        }
        if is_mouse_button_released(MouseButton::Middle) {
            self.m_down = false;
            let chord = self.chord_cell;
            self.chord_cell = -1;
            self.face_down = false;
            if chord >= 0 && self.cell_at(&l, mx, my) == chord {
                self.do_expand(chord as usize);
            }
        }
    }

    pub fn draw(&self) {
        clear_background(C_BTNFACE);
        let l = self.layout();
        self.paint(&l);
        self.draw_menu_bar();

        if self.menu_open == MenuOpen::Game {
            let items = self.game_menu_items();
            // Update mode label dynamically
            let mut display_items: Vec<(&str, i32, bool)> = Vec::new();
            for (label, id, sep) in &items {
                if *id == 150 {
                    let mode_label = if self.game.mode == GameMode::Classic {
                        "Mode: Classic"
                    } else {
                        "Mode: Complex"
                    };
                    display_items.push((Box::leak(mode_label.to_string().into_boxed_str()), *id, *sep));
                } else {
                    display_items.push((*label, *id, *sep));
                }
            }
            self.draw_dropdown(0.0, &display_items, self.menu_hover);
        } else if self.menu_open == MenuOpen::Help {
            let items = App::help_menu_items();
            let game_label = "Game";
            let pad = 12.0;
            let game_w = measure_text(game_label, None, 14, 1.0).width + pad * 2.0;
            self.draw_dropdown(game_w, &items, self.menu_hover);
        }

        // Dialogs on top
        match self.dialog {
            DialogMode::Custom => self.draw_custom_dialog(),
            DialogMode::About => {
                let lines = Self::about_lines();
                let line_refs: Vec<&str> = lines.iter().map(|s| *s).collect();
                self.draw_message_box("About Complexweeper", &line_refs);
            }
            DialogMode::Help => {
                let lines = self.help_lines();
                let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
                self.draw_message_box("How to Play", &line_refs);
            }
            DialogMode::BestScores => {
                let lines = self.best_scores_lines();
                let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
                let title = if self.new_record {
                    "New Record!"
                } else {
                    "Best Scores"
                };
                self.draw_message_box(title, &line_refs);
            }
            DialogMode::None => {}
        }
    }
}
