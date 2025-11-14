use macroquad::camera::Camera2D;
use macroquad::input::{KeyCode, is_key_down};
use macroquad::math::{Rect, Vec2, vec2};
use macroquad::prelude::screen_width;
use macroquad::time::get_frame_time;
use macroquad::window::screen_height;

/// 2D position expressed as horizontal (`x`) and vertical (`y`) components.
#[derive(Debug, Clone)]
pub struct AxisPosition {
    pub x: f32,
    pub y: f32,
}

impl From<Vec2> for AxisPosition {
    fn from(vec: Vec2) -> Self {
        AxisPosition { x: vec.x, y: vec.y }
    }
}

impl From<AxisPosition> for Vec2 {
    fn from(pos: AxisPosition) -> Self {
        vec2(pos.x, pos.y)
    }
}

/// Encapsulates zoom, panning and viewport conversion logic for the editor camera.
#[derive(Debug, Clone)]
pub struct CameraController {
    /// Center point of the camera in world coordinates
    pub screen_center: AxisPosition,
    /// Current zoom level (1.0 = normal, >1.0 = zoomed in, <1.0 = zoomed out)
    pub zoom_level: f32,
}

impl CameraController {
    /// Multiplier used for zoom increment/decrement operations.
    const ZOOM_MULTIPLIER: f32 = 1.2;
    /// Default zoom level when camera is reset.
    const DEFAULT_ZOOM: f32 = 1.0;
    /// Maximum allowed zoom level (upper bound).
    const MAX_ZOOM: f32 = 8.0;
    /// Minimum allowed zoom level (lower bound).
    const MIN_ZOOM: f32 = 0.1;

    const PAN_SPEED: f32 = 1.0;

    /// Creates a controller with the camera centered at the given world position.
    pub fn new(screen_center: AxisPosition) -> Self {
        Self {
            screen_center,
            zoom_level: Self::DEFAULT_ZOOM,
        }
    }

    /// Resets the zoom level to the default value (`1.0`).
    pub fn reset_zoom_level(&mut self) {
        self.zoom_level = Self::DEFAULT_ZOOM;
    }

    /// Decreases the zoom level (zoom out) while respecting the minimum threshold.
    pub fn decrease_zoom_level(&mut self) {
        let zoom_after_decrease: f32 = self.zoom_level / Self::ZOOM_MULTIPLIER;
        self.zoom_level = zoom_after_decrease.max(Self::MIN_ZOOM);
    }

    /// Increases the zoom level (zoom in) while respecting the maximum threshold.
    pub fn increase_zoom_level(&mut self) {
        let zoom_after_increase: f32 = self.zoom_level * Self::ZOOM_MULTIPLIER;
        self.zoom_level = zoom_after_increase.min(Self::MAX_ZOOM);
    }

    /// Returns the current zoom level.
    pub fn get_current_zoom(&self) -> f32 {
        self.zoom_level
    }

    /// Processes keyboard input to update camera position.
    pub fn update_keyboard_pan(&mut self) {
        let mut direction = Vec2::ZERO;

        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            direction.y += 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            direction.y -= 1.0;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            direction.x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            direction.x += 1.0;
        }

        if direction.length_squared() > 0.0 {
            let delta = direction.normalize() * Self::PAN_SPEED * get_frame_time();
            self.screen_center.x += delta.x;
            self.screen_center.y += delta.y;
        }
    }

    /// Calculates the visible world area based on the current zoom level.
    pub fn get_view_size(&self) -> Vec2 {
        let visible_width: f32 = screen_width() / self.zoom_level;
        let visible_height: f32 = screen_height() / self.zoom_level;

        vec2(visible_width, visible_height)
    }

    /// Calculates the boundary position for camera viewport.
    fn calculate_view_bound(center: f32, view_size: f32) -> f32 {
        center - view_size / 2.0
    }

    /// Converts this controller to a Macroquad `Camera2D`.
    pub fn to_camera2d(&self) -> Camera2D {
        let view_size = self.get_view_size();

        let rect = Rect {
            x: Self::calculate_view_bound(self.screen_center.x, view_size.x),
            y: Self::calculate_view_bound(self.screen_center.y, view_size.y),
            w: view_size.x,
            h: view_size.y,
        };

        Camera2D::from_display_rect(rect)
    }

    /// Processes keyboard input to update zoom level.
    pub fn update_zoom_from_input(&mut self) {
        use macroquad::input::is_key_pressed;

        if is_key_pressed(KeyCode::Equal) {
            self.increase_zoom_level();
        }

        if is_key_pressed(KeyCode::Minus) {
            self.decrease_zoom_level();
        }

        if is_key_pressed(KeyCode::Key0) {
            self.reset_zoom_level();
        }
    }

    /// Updates the camera state based on the current grid size.
    pub fn update(&mut self, grid_size: Vec2) {
        self.update_zoom_from_input();
        self.update_keyboard_pan();
        self.clamp_to_bounds(grid_size);
    }

    /// Clamps the camera position to the bounds of the grid.
    fn clamp_to_bounds(&mut self, grid_size: Vec2) {
        let view_size = self.get_view_size();
        self.screen_center.x = clamp_component(self.screen_center.x, grid_size.x, view_size.x);
        self.screen_center.y = clamp_component(self.screen_center.y, grid_size.y, view_size.y);
    }
}

/// Clamps a component of the camera position to the bounds of the grid.
fn clamp_component(center: f32, grid_extent: f32, view_extent: f32) -> f32 {
    let half_view = view_extent / 2.0;
    if grid_extent <= view_extent {
        grid_extent / 2.0
    } else {
        center.clamp(half_view, grid_extent - half_view)
    }
}
