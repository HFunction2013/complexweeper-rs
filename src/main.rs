// Complexweeper — cross-platform minesweeper with Classic and Complex modes.
// Entry point: sets up the macroquad window and runs the game loop.

#![allow(dead_code)]

mod app;
mod assets;
mod game;

#[cfg(test)]
mod game_tests;

use app::App;
use macroquad::window::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Complexweeper".to_string(),
        window_width: 800,
        window_height: 600,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let atlas = match assets::load_builtin() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to load built-in atlas: {}", e);
            std::process::exit(1);
        }
    };

    let mut app = App::new(atlas);

    loop {
        app.update();
        app.draw();
        next_frame().await;
    }
}
