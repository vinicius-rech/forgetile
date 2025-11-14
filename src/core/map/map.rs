use crate::core::map::tile::Size;
use macroquad::camera::{Camera2D, set_camera, set_default_camera};
use macroquad::color::{Color, GRAY, WHITE};
use macroquad::input::{
    KeyCode, MouseButton, is_key_down, is_key_pressed, is_mouse_button_down, mouse_position,
};
use macroquad::math::{Rect, Vec2, vec2};
use macroquad::shapes::{draw_line, draw_rectangle};
use macroquad::texture::{DrawTextureParams, Texture2D, draw_texture_ex};
use macroquad::time::get_frame_time;
use macroquad::window::{screen_height, screen_width};

pub struct Map {
    tile_dimensions: Size,
    zoom_level: f32,
    map_width_tiles: usize,
    map_height_tiles: usize,
    tiles: Vec<Option<PaintedTile>>,
    camera_center: Vec2,
    last_drag_position: Option<Vec2>,
}

#[derive(Clone)]
struct PaintedTile {
    texture: Texture2D,
}

impl Map {
    pub fn new(map_dimension: Size, tile_size: Size) -> Self {
        let map_width_tiles = dimension_to_tiles(map_dimension.width);
        let map_height_tiles = dimension_to_tiles(map_dimension.height);
        let tiles = vec![None; map_width_tiles * map_height_tiles];
        let grid_size = vec2(
            map_width_tiles as f32 * tile_size.width,
            map_height_tiles as f32 * tile_size.height,
        );
        let camera_center = grid_size / 2.0;

        Self {
            tile_dimensions: tile_size,
            zoom_level: 2.0,
            map_width_tiles,
            map_height_tiles,
            tiles,
            camera_center,
            last_drag_position: None,
        }
    }

    pub fn draw(&mut self) -> Camera2D {
        self.update_zoom();
        self.update_camera_pan();
        self.clamp_camera_center();
        let camera = self.setup_camera();
        self.draw_tiles();
        self.setup_grid();
        self.highlight_hovered_tile(&camera);
        set_default_camera();
        camera
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
        let view_size = self.view_size();

        let rect = Rect {
            x: self.camera_center.x - view_size.x / 2.0,
            y: self.camera_center.y - view_size.y / 2.0,
            w: view_size.x,
            h: view_size.y,
        };

        let camera_options = Camera2D::from_display_rect(rect);

        set_camera(&camera_options);
        camera_options
    }

    fn setup_grid(&self) {
        let Size { width: tile_width, height: tile_height } = self.tile_dimensions;

        let grid_width = self.map_width_tiles as f32 * tile_width;
        let grid_height = self.map_height_tiles as f32 * tile_height;
        let thickness = 1.0;

        for col in 0..=self.map_width_tiles {
            let x = col as f32 * tile_width;
            draw_line(x, 0.0, x, grid_height, thickness, GRAY);
        }

        for row in 0..=self.map_height_tiles {
            let y = row as f32 * tile_height;
            draw_line(0.0, y, grid_width, y, thickness, GRAY);
        }
    }

    fn highlight_hovered_tile(&self, camera: &Camera2D) {
        if let Some((tile_x, tile_y)) = self.hovered_tile(camera) {
            let Size { width: tile_width, height: tile_height } = self.tile_dimensions;
            let tile_origin_x = tile_x as f32 * tile_width;
            let tile_origin_y = tile_y as f32 * tile_height;

            let highlight_color = Color { r: 0.1, g: 0.9, b: 0.2, a: 0.35 };
            draw_rectangle(tile_origin_x, tile_origin_y, tile_width, tile_height, highlight_color);
        }
    }

    fn draw_tiles(&self) {
        let tile_width = self.tile_dimensions.width;
        let tile_height = self.tile_dimensions.height;

        for (idx, tile) in self.tiles.iter().enumerate() {
            if let Some(painted) = tile {
                let x = (idx % self.map_width_tiles) as f32 * tile_width;
                let y = (idx / self.map_width_tiles) as f32 * tile_height;
                draw_texture_ex(
                    &painted.texture,
                    x,
                    y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(tile_width, tile_height)),
                        ..Default::default()
                    },
                );
            }
        }
    }

    pub fn hovered_tile(&self, camera: &Camera2D) -> Option<(usize, usize)> {
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_screen = vec2(mouse_x, mouse_y);
        let world_pos = camera.screen_to_world(mouse_screen);

        let tile_width = self.tile_dimensions.width;
        let tile_height = self.tile_dimensions.height;
        let grid_size = self.grid_size();
        let grid_width = grid_size.x;
        let grid_height = grid_size.y;

        if world_pos.x < 0.0
            || world_pos.y < 0.0
            || world_pos.x >= grid_width
            || world_pos.y >= grid_height
        {
            return None;
        }

        let tile_x = (world_pos.x / tile_width).floor() as usize;
        let tile_y = (world_pos.y / tile_height).floor() as usize;

        Some((tile_x, tile_y))
    }

    pub fn paint_tile(&mut self, tile_x: usize, tile_y: usize, texture: Texture2D) {
        if let Some(index) = self.tile_index(tile_x, tile_y) {
            self.tiles[index] = Some(PaintedTile { texture });
        }
    }

    fn tile_index(&self, tile_x: usize, tile_y: usize) -> Option<usize> {
        if tile_x >= self.map_width_tiles || tile_y >= self.map_height_tiles {
            return None;
        }

        Some(tile_y * self.map_width_tiles + tile_x)
    }

    fn update_camera_pan(&mut self) {
        self.update_mouse_pan();
        self.update_keyboard_pan();
    }

    fn update_mouse_pan(&mut self) {
        let (mouse_x, mouse_y) = mouse_position();
        let current = vec2(mouse_x, mouse_y);

        if is_mouse_button_down(MouseButton::Right) {
            if let Some(last) = self.last_drag_position {
                let delta_screen = current - last;
                let delta_world = delta_screen / self.zoom_level;
                if delta_world.length_squared() > 0.0 {
                    self.camera_center += delta_world;
                    self.clamp_camera_center();
                }
            }
            self.last_drag_position = Some(current);
        } else {
            self.last_drag_position = None;
        }
    }

    fn update_keyboard_pan(&mut self) {
        let mut direction = vec2(0.0, 0.0);

        if is_key_down(KeyCode::W) {
            direction.y += 1.0;
        }
        if is_key_down(KeyCode::S) {
            direction.y -= 1.0;
        }
        if is_key_down(KeyCode::A) {
            direction.x -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            direction.x += 1.0;
        }

        if direction.length_squared() > 0.0 {
            let delta = direction.normalize() * 600.0 * get_frame_time();
            self.camera_center += delta;
        }
    }

    fn clamp_camera_center(&mut self) {
        let grid_size = self.grid_size();
        let view_size = self.view_size();

        self.camera_center.x = clamp_component(self.camera_center.x, grid_size.x, view_size.x);
        self.camera_center.y = clamp_component(self.camera_center.y, grid_size.y, view_size.y);
    }

    fn grid_size(&self) -> Vec2 {
        vec2(
            self.map_width_tiles as f32 * self.tile_dimensions.width,
            self.map_height_tiles as f32 * self.tile_dimensions.height,
        )
    }

    fn view_size(&self) -> Vec2 {
        vec2(screen_width() / self.zoom_level, screen_height() / self.zoom_level)
    }
}

fn dimension_to_tiles(value: f32) -> usize {
    value.max(1.0).round() as usize
}

fn clamp_component(center: f32, grid_extent: f32, view_extent: f32) -> f32 {
    let half_view = view_extent / 2.0;
    if grid_extent <= view_extent {
        grid_extent / 2.0
    } else {
        center.clamp(half_view, grid_extent - half_view)
    }
}
