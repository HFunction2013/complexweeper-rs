// Complexweeper — cross-platform minesweeper with Classic, Complex and
// Hyperbolic Complex modes.
// Entry point: sets up the macroquad window and runs the game loop.

#![allow(dead_code)]

mod app;
mod assets;
mod game;
mod sounds;

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

    // Audio is optional: if the backend fails to initialize, the game keeps
    // running silently.
    let sounds = sounds::Sounds::load().await;

    let mut app = App::new(atlas, sounds);

    loop {
        app.update();
        app.draw();
        next_frame().await;
    }
}
