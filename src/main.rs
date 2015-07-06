#![cfg(not(feature="hotloading"))]

extern crate gunship;

pub mod fps;

use gunship::Engine;

pub fn main() {
    let mut engine = Engine::new();
    fps::game_init(&mut engine);
    engine.main_loop();
}
