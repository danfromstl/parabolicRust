use macroquad::prelude::Conf;

mod app;
mod constants;
mod controls;
mod gameplay;
mod hud;
mod input;
mod model;
mod physics;
mod render;
mod state;

fn window_conf() -> Conf {
    app::window_conf()
}

#[macroquad::main(window_conf)]
async fn main() {
    app::run().await;
}
