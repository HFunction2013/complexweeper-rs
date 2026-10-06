// Core game logic — platform-independent.
// Supports three modes: Classic (single mine type, 0-8 clues),
// Complex (four mine types: +1, -1, +i, -i; clues are |sum|^2) and
// Hyperbolic Complex (four mine types: +1, -1, +j, -j with j^2 = +1;
// clues are the signed form magnitude a^2 - b^2).

pub const MAX_W: usize = 40;
pub const MAX_H: usize = 30;
pub const MAX_CELLS: usize = MAX_W * MAX_H;
pub const MAX_MINES: usize = 999;

/// Four complex mine types: (real, imaginary) contribution.
/// Index 0 = +1, 1 = -1, 2 = +i/+j, 3 = -i/-j.
pub const TYPES: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// The 24 achievable display values D = |S|^2 in Complex mode.
pub const ACHIEVABLE: [u16; 24] = [
    0, 1, 2, 4, 5, 8, 9, 10, 13, 16, 17, 18, 20, 25, 26, 29, 32, 34, 36, 37, 40, 49, 50, 64,
];

/// The 39 achievable display values D = a^2 - b^2 in Hyperbolic mode
/// (|a| + |b| <= 8 over an 8-neighborhood). Negative values render as
/// "radical + i unit" (e.g. -4 -> 2i, -7 -> sqrt(7)i).
pub const ACHIEVABLE_HYPER: [i16; 39] = [
    -64, -49, -48, -36, -35, -32, -25, -24, -21, -16, -15, -12, -9, -8, -7, -5, -4, -3, -1, 0, 1,
    3, 4, 5, 7, 8, 9, 12, 15, 16, 21, 24, 25, 32, 35, 36, 48, 49, 64,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Classic,
    Complex,
    Hyper,
}

#[derive(Debug, Clone)]
pub struct Preset {
    pub w: u16,
    pub h: u16,
    pub mines: u16,
    pub label: &'static str,
}

/// Standard three presets (mines scattered randomly; in Complex mode types are random too).
pub const PRESETS: [Preset; 3] = [
    Preset {
        w: 9,
        h: 9,
        mines: 10,
        label: "Beginner 9x9 - 10 mines",
    },
    Preset {
        w: 16,
        h: 16,
        mines: 40,
        label: "Intermediate 16x16 - 40 mines",
    },
    Preset {
        w: 30,
        h: 16,
        mines: 99,
        label: "Expert 30x16 - 99 mines",
    },
];

/// Status message enum (UI layer provides the text).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    None,
    Started,
    JudgeFail,
    ExpandOk,
    Win,
    Lose,
}

/// mulberry32 PRNG — same algorithm as the original, same seed = same board.
#[derive(Debug, Clone)]
pub struct Rng {
    a: u32,
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng {
            a: if seed == 0 { 1 } else { seed },
        }
    }

    pub fn next_f64(&mut self) -> f64 {
        self.a = self.a.wrapping_add(0x6D2B79F5);
        let mut t: u32 = self.a;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        (t ^ (t >> 14)) as f64 / 4294967296.0
    }

    /// Integer in [0, n).
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_f64() * n as f64) as usize
    }
}

pub struct Game {
    pub mode: GameMode,
    pub w: u16,
    pub h: u16,
    pub n: usize,
    /// Mine type per cell: 0 = no mine, 1..4 = mine type (Complex);
    /// in Classic mode, mines are always type 1.
    pub mine: [u8; MAX_CELLS],
    /// Clue value: -1 = uncomputed / mine cell.
    /// Classic: 0..8 neighbor mine count. Complex: |sum|^2 (one of ACHIEVABLE).
    pub clue: [i16; MAX_CELLS],
    pub open: [u8; MAX_CELLS],
    /// Flag per cell: 0 = none, 1..4 = flag type. In Classic mode only 0 or 1.
    pub flag: [u8; MAX_CELLS],
    pub seed: u32,
    pub rng: Rng,
    pub mines: u16,
    /// Total count of each mine type (indices 1..4), known after generation.
    pub type_total: [u16; 5],
    /// Current flag count per type (indices 1..4).
    pub flags_of: [u16; 5],
    /// Custom type ratio (indices 1..4); all zero = "random types".
    pub type_count: [u16; 5],
    pub started: bool,
    pub over: bool,
    pub win: bool,
    pub boom: i32,
    pub start_cell: i32,
    pub elapsed_ms: u32,
    pub t0: u32,
    pub moves: u32,
    pub msg: Msg,
    pub msg_arg: u16,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            mode: GameMode::Complex,
            w: 9,
            h: 9,
            n: 81,
            mine: [0; MAX_CELLS],
            clue: [-1; MAX_CELLS],
            open: [0; MAX_CELLS],
            flag: [0; MAX_CELLS],
            seed: 1,
            rng: Rng::new(1),
            mines: 10,
            type_total: [0; 5],
            flags_of: [0; 5],
            type_count: [0; 5],
            started: false,
            over: false,
            win: false,
            boom: -1,
            start_cell: -1,
            elapsed_ms: 0,
            t0: 0,
            moves: 0,
            msg: Msg::None,
            msg_arg: 0,
        }
    }
}

impl Game {
    pub fn new(mode: GameMode) -> Self {
        Game {
            mode,
            ..Default::default()
        }
    }

    pub fn in_bounds(&self, r: i32, c: i32) -> bool {
        r >= 0 && c >= 0 && r < self.h as i32 && c < self.w as i32
    }

    /// 8-neighborhood; writes into buf, returns count.
    pub fn neighbors(&self, cell: usize, buf: &mut [usize; 8]) -> usize {
        let w = self.w as i32;
        let r = (cell / self.w as usize) as i32;
        let c = (cell % self.w as usize) as i32;
        let mut k = 0;
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let rr = r + dr;
                let cc = c + dc;
                if rr < 0 || cc < 0 || rr >= self.h as i32 || cc >= self.w as i32 {
                    continue;
                }
                buf[k] = (rr * w + cc) as usize;
                k += 1;
            }
        }
        k
    }

    pub fn neighbor_mine_count(&self, cell: usize) -> usize {
        let mut buf = [0usize; 8];
        let k = self.neighbors(cell, &mut buf);
        let mut n = 0;
        for &j in &buf[..k] {
            if self.mine[j] != 0 {
                n += 1;
            }
        }
        n
    }

    pub fn neighbor_flag_count(&self, cell: usize) -> usize {
        let mut buf = [0usize; 8];
        let k = self.neighbors(cell, &mut buf);
        let mut n = 0;
        for &j in &buf[..k] {
            if self.flag[j] != 0 {
                n += 1;
            }
        }
        n
    }

    /// Blank cell: no mines in neighborhood. Only blank cells cascade.
    pub fn is_blank(&self, cell: usize) -> bool {
        self.mine[cell] == 0 && self.neighbor_mine_count(cell) == 0
    }

    pub fn flags_total(&self) -> usize {
        let mut s = 0usize;
        for t in 1..5 {
            s += self.flags_of[t] as usize;
        }
        s
    }

    /// How many mines of type t are still unmarked (can be negative).
    pub fn unmarked(&self, t: usize) -> i32 {
        self.type_total[t] as i32 - self.flags_of[t] as i32
    }

    pub fn set_msg(&mut self, m: Msg) {
        self.msg = m;
        self.msg_arg = 0;
    }

    // ------------------------------------------------------------------ generation

    pub fn set_seed(&mut self, s: u32) {
        self.seed = if s == 0 { 1 } else { s };
        self.rng = Rng::new(self.seed);
    }

    /// Start a new game (board not yet generated; waits for first click).
    pub fn new_game(&mut self, seed: u32) {
        self.n = self.w as usize * self.h as usize;
        for i in 0..self.n {
            self.mine[i] = 0;
            self.clue[i] = -1;
            self.open[i] = 0;
            self.flag[i] = 0;
        }
        self.started = false;
        self.over = false;
        self.win = false;
        self.boom = -1;
        self.start_cell = -1;
        self.elapsed_ms = 0;
        self.t0 = 0;
        self.moves = 0;
        self.type_total = [0; 5];
        self.flags_of = [0; 5];
        self.set_seed(seed);
        self.set_msg(Msg::None);
    }

    /// Generate board + compute clues + cascade from start cell.
    /// Safe zone = start cell and its in-bounds 8 neighbors.
    pub fn gen_board(&mut self, start_cell: usize) {
        let n = self.n;
        for i in 0..n {
            self.mine[i] = 0;
            self.clue[i] = -1;
            self.open[i] = 0;
            // Note: do NOT clear flag[] here. Flags placed before first click
            // survive generation (classic minesweeper behavior).
        }
        let mut is_safe = [false; MAX_CELLS];
        is_safe[start_cell] = true;
        let mut nbuf = [0usize; 8];
        let nk = self.neighbors(start_cell, &mut nbuf);
        for &j in &nbuf[..nk] {
            is_safe[j] = true;
        }

        let mut pool = [0usize; MAX_CELLS];
        let mut m = 0usize;
        for (i, &safe) in is_safe[..n].iter().enumerate() {
            if !safe {
                pool[m] = i;
                m += 1;
            }
        }
        // Shuffle positions (Fisher-Yates).
        if m > 1 {
            let mut i = m - 1;
            while i > 0 {
                let j = self.rng.below(i + 1);
                pool.swap(i, j);
                i -= 1;
            }
        }

        let mut want: usize = 0;
        for t in 1..5 {
            want += self.type_count[t] as usize;
        }
        let count = if want > 0 { want } else { self.mines as usize }.min(m);

        if self.mode == GameMode::Classic {
            // Classic: all mines are type 1.
            for &idx in pool.iter().take(count) {
                self.mine[idx] = 1;
            }
        } else if want > 0 {
            // Complex with exact ratio: build type list, shuffle, place.
            let mut list = [0u8; MAX_CELLS];
            let mut ln = 0usize;
            for t in 1..5 {
                for _ in 0..self.type_count[t] {
                    list[ln] = t as u8;
                    ln += 1;
                }
            }
            if ln > count {
                ln = count;
            }
            if ln > 1 {
                let mut i = ln - 1;
                while i > 0 {
                    let j = self.rng.below(i + 1);
                    list.swap(i, j);
                    i -= 1;
                }
            }
            for k in 0..ln {
                self.mine[pool[k]] = list[k];
            }
        } else {
            // Complex random types.
            for &idx in pool.iter().take(count) {
                self.mine[idx] = (1 + self.rng.below(4)) as u8;
            }
        }
        self.mines = count as u16;
        self.compute_clues();
        self.count_types();
        let seeds = [start_cell];
        self.cascade_open(&seeds);
    }

    pub fn compute_clues(&mut self) {
        for i in 0..self.n {
            if self.mine[i] != 0 {
                self.clue[i] = -1;
                continue;
            }
            if self.mode == GameMode::Classic {
                self.clue[i] = self.neighbor_mine_count(i) as i16;
            } else {
                let mut a: i32 = 0;
                let mut b: i32 = 0;
                let mut buf = [0usize; 8];
                let k = self.neighbors(i, &mut buf);
                for &j in &buf[..k] {
                    if self.mine[j] == 0 {
                        continue;
                    }
                    let t = TYPES[(self.mine[j] - 1) as usize];
                    a += t.0;
                    b += t.1;
                }
                self.clue[i] = if self.mode == GameMode::Hyper {
                    // Hyperbolic form magnitude: a^2 - b^2 (can be negative).
                    (a * a - b * b) as i16
                } else {
                    (a * a + b * b) as i16
                };
            }
        }
    }

    pub fn count_types(&mut self) {
        self.type_total = [0; 5];
        for i in 0..self.n {
            if self.mine[i] != 0 {
                self.type_total[self.mine[i] as usize] += 1;
            }
        }
    }

    /// Cascade-open from seeds; only blank cells propagate. Returns newly opened count.
    pub fn cascade_open(&mut self, seeds: &[usize]) -> usize {
        let mut stack = [0usize; MAX_CELLS];
        let mut queued = [false; MAX_CELLS];
        let mut sp = 0usize;
        for &s in seeds {
            if !queued[s] {
                queued[s] = true;
                stack[sp] = s;
                sp += 1;
            }
        }
        let mut opened = 0usize;
        while sp > 0 {
            sp -= 1;
            let i = stack[sp];
            if self.open[i] != 0 || self.mine[i] != 0 || self.flag[i] != 0 {
                continue;
            }
            self.open[i] = 1;
            opened += 1;
            if self.is_blank(i) {
                let mut buf = [0usize; 8];
                let k = self.neighbors(i, &mut buf);
                for &j in &buf[..k] {
                    if !queued[j] && self.open[j] == 0 && self.mine[j] == 0 && self.flag[j] == 0 {
                        queued[j] = true;
                        stack[sp] = j;
                        sp += 1;
                    }
                }
            }
        }
        opened
    }

    // ------------------------------------------------------------------ actions

    pub fn start_at(&mut self, cell: usize, now_ms: u32) {
        self.set_seed(self.seed);
        self.gen_board(cell);
        self.start_cell = cell as i32;
        self.started = true;
        self.over = false;
        self.win = false;
        self.elapsed_ms = 0;
        self.t0 = now_ms;
        self.moves = 1;
        self.set_msg(Msg::None);
    }

    /// Set/change/clear a flag. Always succeeds (flags are unlimited).
    /// In Classic mode, t is clamped to 0 or 1.
    pub fn set_flag(&mut self, cell: usize, t: u8) -> bool {
        let t = if self.mode == GameMode::Classic && t > 1 {
            1
        } else if t > 4 {
            return false;
        } else {
            t
        };
        let old = self.flag[cell];
        if old == t {
            return true;
        }
        if old != 0 {
            self.flags_of[old as usize] -= 1;
        }
        self.flag[cell] = t;
        if t != 0 {
            self.flags_of[t as usize] += 1;
        }
        true
    }

    /// Right-click cycle. Classic: none -> flag -> none.
    /// Complex: none -> +1 -> -1 -> +i -> -i -> none.
    pub fn cycle_flag(&mut self, cell: usize) -> bool {
        if self.over || self.open[cell] != 0 {
            return false;
        }
        let max = if self.mode == GameMode::Classic { 2 } else { 5 };
        let next = ((self.flag[cell] as usize) + 1) % max;
        self.set_flag(cell, next as u8);
        self.moves += 1;
        true
    }

    /// Reveal one cell. Flagged cells cannot be revealed (classic behavior).
    pub fn reveal(&mut self, cell: usize, _now_ms: u32) {
        if self.over || self.open[cell] != 0 || self.flag[cell] != 0 {
            return;
        }
        if self.mine[cell] != 0 {
            self.open[cell] = 1;
            self.lose(cell);
            return;
        }
        self.open[cell] = 1;
        if self.is_blank(cell) {
            let mut buf = [0usize; 8];
            let k = self.neighbors(cell, &mut buf);
            let slice: Vec<usize> = buf[..k].to_vec();
            self.cascade_open(&slice);
        }
        self.moves += 1;
        self.check_win();
    }

    /// Chord matching criterion.
    /// Classic: total flags == total mines in neighborhood.
    /// Complex: total flags == total mines, AND real/imaginary flag counts match
    /// (or are swapped, since the clue is symmetric).
    /// Hyper: the display value a^2 - b^2 hides the individual signs of a and b
    /// (swapping a and b would change the sign of D), so the criterion is that
    /// |a| and |b| each match — i.e. any of the four sign combinations is fine.
    pub fn match_combo_truth(&self, cell: usize) -> bool {
        if self.mode == GameMode::Classic {
            let mut truth = 0u16;
            let mut got = 0u16;
            let mut buf = [0usize; 8];
            let k = self.neighbors(cell, &mut buf);
            for &j in &buf[..k] {
                if self.mine[j] != 0 {
                    truth += 1;
                }
                if self.flag[j] != 0 {
                    got += 1;
                }
            }
            return got == truth;
        }

        if self.mode == GameMode::Hyper {
            // Flag count must match mine count first.
            if self.neighbor_mine_count(cell) != self.neighbor_flag_count(cell) {
                return false;
            }
            let t = self.sums_of(cell, false);
            let f = self.sums_of(cell, true);
            return f.0.abs() == t.0.abs() && f.1.abs() == t.1.abs();
        }

        let mut truth = [0u16; 4];
        let mut got = [0u16; 4];
        let mut buf = [0usize; 8];
        let k = self.neighbors(cell, &mut buf);
        for &j in &buf[..k] {
            if self.mine[j] != 0 {
                truth[(self.mine[j] - 1) as usize] += 1;
            }
            if self.flag[j] != 0 {
                got[(self.flag[j] - 1) as usize] += 1;
            }
        }
        let p = truth[0] + truth[1];
        let v = truth[2] + truth[3];
        let gp = got[0] + got[1];
        let gv = got[2] + got[3];
        (gp + gv == p + v) && ((gp == p && gv == v) || (gp == v && gv == p))
    }

    /// Signed sums of the real and imaginary (i / j) parts of the mines
    /// (`use_flag == false`) or flags (`use_flag == true`) in the neighborhood.
    /// Returns (real_sum, imag_sum).
    fn sums_of(&self, cell: usize, use_flag: bool) -> (i32, i32) {
        let mut buf = [0usize; 8];
        let k = self.neighbors(cell, &mut buf);
        let mut a = 0i32;
        let mut b = 0i32;
        for &j in &buf[..k] {
            let t: u8 = if use_flag {
                self.flag[j]
            } else {
                self.mine[j]
            };
            if t == 0 {
                continue;
            }
            let (ra, rb) = TYPES[(t - 1) as usize];
            a += ra;
            b += rb;
        }
        (a, b)
    }

    /// Chord / double-click expand: if criterion passes, reveal unflagged neighbors.
    pub fn try_expand(&mut self, cell: usize) {
        if self.over || self.open[cell] == 0 || self.mine[cell] != 0 {
            return;
        }
        let mut buf = [0usize; 8];
        let k = self.neighbors(cell, &mut buf);
        let mut uns = [0usize; 8];
        let mut un = 0usize;
        for &j in &buf[..k] {
            if self.open[j] == 0 && self.flag[j] == 0 {
                uns[un] = j;
                un += 1;
            }
        }
        if un == 0 {
            return;
        }
        if !self.match_combo_truth(cell) {
            self.set_msg(Msg::JudgeFail);
            return;
        }
        let mut boom: i32 = -1;
        for &j in &uns[..un] {
            if self.mine[j] != 0 {
                boom = j as i32;
                break;
            }
        }
        if boom >= 0 {
            let b = boom as usize;
            self.open[b] = 1;
            self.lose(b);
            return;
        }
        let slice: Vec<usize> = uns[..un].to_vec();
        self.cascade_open(&slice);
        self.moves += 1;
        self.msg_arg = un as u16;
        self.set_msg(Msg::ExpandOk);
        self.check_win();
    }

    pub fn check_win(&mut self) {
        for i in 0..self.n {
            if self.mine[i] == 0 && self.open[i] == 0 {
                return;
            }
        }
        self.over = true;
        self.win = true;
        self.set_msg(Msg::Win);
    }

    pub fn lose(&mut self, cell: usize) {
        self.over = true;
        self.win = false;
        self.boom = cell as i32;
        self.set_msg(Msg::Lose);
    }

    pub fn opened_count(&self) -> usize {
        let mut k = 0usize;
        for i in 0..self.n {
            if self.open[i] != 0 {
                k += 1;
            }
        }
        k
    }

    pub fn safe_count(&self) -> usize {
        let mut k = 0usize;
        for i in 0..self.n {
            if self.mine[i] == 0 {
                k += 1;
            }
        }
        k
    }

    pub fn correct_flags(&self) -> usize {
        let mut k = 0usize;
        for i in 0..self.n {
            if self.mine[i] != 0 && self.flag[i] == self.mine[i] {
                k += 1;
            }
        }
        k
    }

    pub fn type_sum(&self) -> usize {
        let mut s = 0usize;
        for t in 1..5 {
            s += self.type_total[t] as usize;
        }
        s
    }
}

/// Split a total evenly across four mine types (for custom dialog pre-fill).
pub fn split_evenly(total: u16) -> [u16; 5] {
    let mut out = [0u16; 5];
    let base = total / 4;
    let mut rest = total - base * 4;
    for (_, slot) in out.iter_mut().enumerate().skip(1).take(4) {
        *slot = base;
        if rest > 0 {
            *slot += 1;
            rest -= 1;
        }
    }
    out
}
