#![allow(dead_code)]
#![warn(clippy::pedantic)]

mod game;
use game::*;

mod window;
use window::*;

mod vulkan;
use vulkan::*;

mod types;

fn main() {
    let mut game = game::Game::new();

    game.game_loop();
}
