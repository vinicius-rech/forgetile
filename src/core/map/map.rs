use crate::core::map::tile::Size;
use macroquad::camera::{Camera2D, set_camera, set_default_camera};
use macroquad::color::{Color, GRAY};
use macroquad::input::{KeyCode, is_key_pressed, mouse_position};
use macroquad::math::{Rect, vec2};
use macroquad::shapes::{draw_line, draw_rectangle};
use macroquad::window::{screen_height, screen_width};

pub struct Map {
    map_dimension: Size,
    tile_dimensions: Size,
    zoom_level: f32,
}

impl Map {
    pub fn new(map_dimension: Size, tile_size: Size) -> Self {
        Self {
            map_dimension,
            tile_dimensions: tile_size,
            zoom_level: 2.0,
        }
    }

    pub fn draw(&mut self) {
        self.update_zoom();
        let camera = self.setup_camera();
        self.setup_grid();
        self.highlight_hovered_tile(&camera);
        set_default_camera();
    }

    pub fn current_zoom_level(&self) -> f32 {
        self.zoom_level
    }

    pub fn update_zoom(&mut self) {
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            self.zoom_level = (self.zoom_level * 1.2).min(8.0);
            println!("🔍 ZOOM IN: {:.1}x", self.zoom_level);
        }

        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            self.zoom_level = (self.zoom_level / 1.2).max(0.5);
            println!("🔍 ZOOM OUT: {:.1}x", self.zoom_level);
        }

        if is_key_pressed(KeyCode::Key0) || is_key_pressed(KeyCode::Kp0) {
            self.zoom_level = 2.0;
            println!("🔍 ZOOM RESET: {:.1}x", self.zoom_level);
        }
    }

    fn setup_camera(&self) -> Camera2D {
        let Size { width: map_width, height: map_height } = self.map_dimension;
        let Size { width: tile_width, height: tile_height } = self.tile_dimensions;

        let grid_width: f32 = map_width * tile_width;
        let grid_height: f32 = map_height * tile_height;
        let grid_center = vec2(grid_width / 2.0, grid_height / 2.0);

        let view_width = screen_width() / self.zoom_level;
        let view_height = screen_height() / self.zoom_level;

        let rect = Rect {
            x: grid_center.x - view_width / 2.0,
            y: grid_center.y - view_height / 2.0,
            w: view_width,
            h: view_height,
        };

        let camera_options = Camera2D::from_display_rect(rect);

        set_camera(&camera_options);
        camera_options
    }

    fn setup_grid(&self) {
        let map_width_tiles = self.map_dimension.width as usize;
        let map_height_tiles = self.map_dimension.height as usize;
        let Size { width: tile_width, height: tile_height } = self.tile_dimensions;

        let grid_width = map_width_tiles as f32 * tile_width;
        let grid_height = map_height_tiles as f32 * tile_height;
        let thickness = 1.0;

        for col in 0..=map_width_tiles {
            let x = col as f32 * tile_width;
            draw_line(x, 0.0, x, grid_height, thickness, GRAY);
        }

        for row in 0..=map_height_tiles {
            let y = row as f32 * tile_height;
            draw_line(0.0, y, grid_width, y, thickness, GRAY);
        }
    }

    fn highlight_hovered_tile(&self, camera: &Camera2D) {
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_screen = vec2(mouse_x, mouse_y);
        let world_pos = camera.screen_to_world(mouse_screen);

        let Size { width: tile_width, height: tile_height } = self.tile_dimensions;
        let grid_width = self.map_dimension.width * tile_width;
        let grid_height = self.map_dimension.height * tile_height;

        if world_pos.x < 0.0
            || world_pos.y < 0.0
            || world_pos.x >= grid_width
            || world_pos.y >= grid_height
        {
            return;
        }

        let tile_x = (world_pos.x / tile_width).floor();
        let tile_y = (world_pos.y / tile_height).floor();

        let tile_origin_x = tile_x * tile_width;
        let tile_origin_y = tile_y * tile_height;

        let highlight_color = Color { r: 0.1, g: 0.9, b: 0.2, a: 0.35 };
        draw_rectangle(tile_origin_x, tile_origin_y, tile_width, tile_height, highlight_color);
    }
}
