use crate::core::map::tile::{Dimensions, Tile};
use macroquad::camera::{Camera2D, set_camera, set_default_camera};
use macroquad::color::GRAY;
use macroquad::math::vec2;
use macroquad::shapes::draw_rectangle_lines;
use macroquad::window::{screen_height, screen_width};

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
        let grid_width = self.map_dimension.width as f32 * self.tile_dimensions.width as f32;
        let grid_height = self.map_dimension.height as f32 * self.tile_dimensions.height as f32;

        let grid_center_x = grid_width / 2.0;
        let grid_center_y = grid_height / 2.0;

        set_camera(&Camera2D {
            target: vec2(grid_center_x, grid_center_y),
            zoom: vec2(1.0 / screen_width() * 2.0, 1.0 / screen_height() * 2.0),
            ..Default::default()
        });

        for row in 0..self.map_dimension.height {
            for col in 0..self.map_dimension.width {
                let x = col as f32 * self.tile_dimensions.width as f32;
                let y = row as f32 * self.tile_dimensions.height as f32;

                draw_rectangle_lines(
                    x,
                    y,
                    self.tile_dimensions.width as f32,
                    self.tile_dimensions.height as f32,
                    1.0,
                    GRAY,
                );
            }
        }

        set_default_camera();
    }
}
