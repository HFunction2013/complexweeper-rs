// Unit tests for core game logic.

#[cfg(test)]
mod tests {
    use crate::game::*;

    #[test]
    fn test_rng_deterministic() {
        let mut r1 = Rng::new(42);
        let mut r2 = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(r1.next_f64(), r2.next_f64());
        }
    }

    #[test]
    fn test_rng_below() {
        let mut r = Rng::new(123);
        for _ in 0..1000 {
            let v = r.below(10);
            assert!(v < 10);
        }
    }

    #[test]
    fn test_classic_new_game() {
        let mut g = Game::new(GameMode::Classic);
        g.w = 9;
        g.h = 9;
        g.mines = 10;
        g.new_game(1);
        assert_eq!(g.n, 81);
        assert!(!g.started);
        assert!(!g.over);
        assert_eq!(g.opened_count(), 0);
    }

    #[test]
    fn test_classic_first_click_safe() {
        let mut g = Game::new(GameMode::Classic);
        g.w = 9;
        g.h = 9;
        g.mines = 10;
        g.new_game(1);
        let start = 4 * 9 + 4; // center
        g.start_at(start, 0);
        assert!(g.started);
        // Start cell and neighbors must be safe
        assert_eq!(g.mine[start], 0);
        let mut buf = [0usize; 8];
        let k = g.neighbors(start, &mut buf);
        for &j in &buf[..k] {
            assert_eq!(g.mine[j], 0, "neighbor {} should be safe", j);
        }
        // Start cell should be opened
        assert_eq!(g.open[start], 1);
    }

    #[test]
    fn test_classic_clues() {
        let mut g = Game::new(GameMode::Classic);
        g.w = 5;
        g.h = 5;
        g.mines = 3;
        g.new_game(1);
        g.start_at(12, 0); // center
        g.compute_clues();
        // All non-mine cells should have clues 0-8
        for i in 0..g.n {
            if g.mine[i] == 0 {
                assert!(
                    g.clue[i] >= 0 && g.clue[i] <= 8,
                    "cell {} clue {}",
                    i,
                    g.clue[i]
                );
            } else {
                assert_eq!(g.clue[i], -1);
            }
        }
    }

    #[test]
    fn test_classic_mine_count() {
        let mut g = Game::new(GameMode::Classic);
        g.w = 9;
        g.h = 9;
        g.mines = 10;
        g.new_game(1);
        g.start_at(40, 0);
        let actual: usize = (0..g.n).filter(|&i| g.mine[i] != 0).count();
        assert_eq!(actual, 10);
    }

    #[test]
    fn test_complex_four_types() {
        let mut g = Game::new(GameMode::Complex);
        g.w = 16;
        g.h = 16;
        g.mines = 40;
        g.new_game(1);
        g.start_at(8 * 16 + 8, 0);
        // Count types
        let mut types = [0u16; 5];
        for i in 0..g.n {
            if g.mine[i] != 0 {
                types[g.mine[i] as usize] += 1;
            }
        }
        let total: u16 = types[1..5].iter().sum();
        assert_eq!(total, 40);
        // In random mode, should have some of each type (with 40 mines, very likely)
        assert!(types[1] > 0);
        assert!(types[2] > 0);
        assert!(types[3] > 0);
        assert!(types[4] > 0);
    }

    #[test]
    fn test_complex_clue_achievable() {
        let mut g = Game::new(GameMode::Complex);
        g.w = 16;
        g.h = 16;
        g.mines = 40;
        g.new_game(1);
        g.start_at(136, 0);
        for i in 0..g.n {
            if g.mine[i] == 0 && g.clue[i] >= 0 {
                assert!(
                    ACHIEVABLE.contains(&(g.clue[i] as u16)),
                    "cell {} has non-achievable clue {}",
                    i,
                    g.clue[i]
                );
            }
        }
    }

    #[test]
    fn test_flag_cycle_classic() {
        let mut g = Game::new(GameMode::Classic);
        g.w = 5;
        g.h = 5;
        g.mines = 3;
        g.new_game(1);
        g.start_at(12, 0);
        // Find a closed cell
        let cell = (0..g.n).find(|&i| g.open[i] == 0).unwrap();
        assert_eq!(g.flag[cell], 0);
        g.cycle_flag(cell);
        assert_eq!(g.flag[cell], 1);
        g.cycle_flag(cell);
        assert_eq!(g.flag[cell], 0);
    }

    #[test]
    fn test_flag_cycle_complex() {
        let mut g = Game::new(GameMode::Complex);
        g.w = 5;
        g.h = 5;
        g.mines = 3;
        g.new_game(1);
        g.start_at(12, 0);
        let cell = (0..g.n).find(|&i| g.open[i] == 0).unwrap();
        assert_eq!(g.flag[cell], 0);
        g.cycle_flag(cell);
        assert_eq!(g.flag[cell], 1);
        g.cycle_flag(cell);
        assert_eq!(g.flag[cell], 2);
        g.cycle_flag(cell);
        assert_eq!(g.flag[cell], 3);
        g.cycle_flag(cell);
        assert_eq!(g.flag[cell], 4);
        g.cycle_flag(cell);
        assert_eq!(g.flag[cell], 0);
    }

    #[test]
    fn test_cascade_open() {
        let mut g = Game::new(GameMode::Classic);
        g.w = 9;
        g.h = 9;
        g.mines = 10;
        g.new_game(1);
        let start = 40;
        g.start_at(start, 0);
        // After start, at least the start cell is open
        assert!(g.opened_count() >= 1);
        // Blank cells should cascade
    }

    #[test]
    fn test_split_evenly() {
        let r = split_evenly(10);
        assert_eq!(r[1] + r[2] + r[3] + r[4], 10);
        // Should be as even as possible: 3,3,2,2
        assert!(r[1] >= 2 && r[1] <= 3);

        let r = split_evenly(0);
        assert_eq!(r[1] + r[2] + r[3] + r[4], 0);

        let r = split_evenly(1);
        assert_eq!(r[1] + r[2] + r[3] + r[4], 1);
    }

    #[test]
    fn test_custom_type_ratio() {
        let mut g = Game::new(GameMode::Complex);
        g.w = 10;
        g.h = 10;
        g.mines = 20;
        g.type_count = [0, 5, 5, 5, 5]; // exact ratio
        g.new_game(1);
        g.start_at(55, 0);
        assert_eq!(g.type_total[1], 5);
        assert_eq!(g.type_total[2], 5);
        assert_eq!(g.type_total[3], 5);
        assert_eq!(g.type_total[4], 5);
    }

    #[test]
    fn test_win_condition() {
        let mut g = Game::new(GameMode::Classic);
        g.w = 5;
        g.h = 5;
        g.mines = 3;
        g.new_game(1);
        g.start_at(12, 0); // center
                           // Reveal all non-mine cells
        for i in 0..g.n {
            if g.mine[i] == 0 && g.open[i] == 0 {
                g.reveal(i, 0);
            }
        }
        assert!(g.over);
        assert!(g.win);
    }

    #[test]
    fn test_lose_condition() {
        let mut g = Game::new(GameMode::Classic);
        g.w = 5;
        g.h = 5;
        g.mines = 5;
        g.new_game(1);
        g.start_at(12, 0);
        // Find a mine
        let mine_cell = (0..g.n).find(|&i| g.mine[i] != 0).unwrap();
        g.reveal(mine_cell, 0);
        assert!(g.over);
        assert!(!g.win);
        assert_eq!(g.boom, mine_cell as i32);
    }

    #[test]
    fn test_flagged_cell_cannot_reveal() {
        let mut g = Game::new(GameMode::Classic);
        g.w = 5;
        g.h = 5;
        g.mines = 3;
        g.new_game(1);
        g.start_at(12, 0);
        let cell = (0..g.n)
            .find(|&i| g.open[i] == 0 && g.mine[i] == 0)
            .unwrap();
        g.set_flag(cell, 1);
        g.reveal(cell, 0);
        assert_eq!(g.open[cell], 0); // still closed
    }

    #[test]
    fn test_neighbors() {
        let g = Game::new(GameMode::Classic);
        // Corner cell
        let mut buf = [0usize; 8];
        // Can't call neighbors on default game (w=9,h=9,n=81)
        let k = g.neighbors(0, &mut buf);
        assert_eq!(k, 3); // top-left corner has 3 neighbors
        let k = g.neighbors(4, &mut buf);
        assert_eq!(k, 5); // top edge has 5 neighbors
        let k = g.neighbors(40, &mut buf);
        assert_eq!(k, 8); // center has 8 neighbors
    }
}
