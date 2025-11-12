use crate::core::map::tile::{Dimensions, Tile};
use macroquad::color::YELLOW;
use macroquad::shapes::{draw_rectangle, draw_rectangle_lines};
use macroquad::window::screen_width;

pub struct Map {
    map_dimension: Dimensions,
    tile_dimensions: Dimensions,
}

impl Map {
    pub fn new(map_dimension: Dimensions, tile_size: Dimensions) -> Self {
        Self {
            map_dimension,
            tile_dimensions: tile_size,
        }
    }

    pub fn init(&self) {
        self.generate_grid();
    }

    fn generate_grid(&self) {
        let half_tile_width = self.tile_dimensions.width / 2;
        let half_tile_height = self.tile_dimensions.height / 2;

        let screen_center_x: f32 = (screen_width() / 2.0) - half_tile_width as f32;
        let screen_center_y: f32 = (screen_width() / 2.0) - half_tile_height as f32;

        for row in 0..self.map_dimension.height as i32 {
            for col in 0..self.map_dimension.width as i32 {
                let x: f32 = screen_center_x + col as f32 * self.tile_dimensions.width as f32;
                let y: f32 = screen_center_y + row as f32 * self.tile_dimensions.height as f32;

                draw_rectangle_lines(
                    x,
                    y,
                    self.tile_dimensions.width as f32,
                    self.tile_dimensions.height as f32,
                    1.0,
                    YELLOW,
                );
            }
        }
    }
}
