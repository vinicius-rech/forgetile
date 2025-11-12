use crate::core::map::map::Map;
use crate::core::map::tile::Size;
use macroquad::color::{BLACK, DARKGRAY, WHITE};
use macroquad::prelude::clear_background;
use macroquad::text::draw_text;
use macroquad::window::next_frame;

mod core;

#[macroquad::main("ForgeTile")]
async fn main() {
    let map_dimension = Size { width: 100.0, height: 100.0 };
    let tile_size = Size { width: 32.0, height: 32.0 };
    let mut map = Map::new(map_dimension, tile_size);

    loop {
        clear_background(BLACK);

        draw_text("ForgeTile!", 20.0, 20.0, 30.0, DARKGRAY);

        map.draw();

        draw_text(
            &format!(
                "Zoom: {:.1}x | Controls: +/- to zoom | 0 to reset",
                map.current_zoom_level()
            ),
            10.0,
            50.0,
            20.0,
            WHITE,
        );

        next_frame().await;
    }
}
