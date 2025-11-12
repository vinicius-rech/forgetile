use crate::core::map::map::Map;
use crate::core::map::tile::Dimensions;
use macroquad::color::{BLACK, DARKGRAY};
use macroquad::prelude::clear_background;
use macroquad::text::draw_text;
use macroquad::window::next_frame;

mod core;

#[macroquad::main("ForgeTile")]
async fn main() {
    loop {
        clear_background(BLACK);

        draw_text("ForgeTile!", 20.0, 20.0, 30.0, DARKGRAY);

        let map_dimension = Dimensions { width: 100, height: 100 };

        let tile_size = Dimensions { width: 32, height: 32 };

        let map = Map::new(map_dimension, tile_size);

        map.init();

        next_frame().await;
    }
}
