use crate::core::assets::{AssetCatalog, TileSprite};
use crate::core::camera::{AxisPosition, CameraController};
use crate::core::map::tile::Size;
use macroquad::camera::{Camera2D, set_camera, set_default_camera};
use macroquad::color::{Color, GRAY, WHITE};
use macroquad::input::mouse_position;
use macroquad::math::{Vec2, vec2};
use macroquad::shapes::{draw_line, draw_rectangle};
use macroquad::texture::{DrawTextureParams, Texture2D, draw_texture_ex};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

/// Runtime representation of the editable tile map.
pub struct Map {
    camera_controller: CameraController,
    tiles: Vec<Option<PaintedTile>>,
    map_height_tiles: usize,
    map_width_tiles: usize,
    tile_dimensions: Size,
}

#[derive(Clone)]
struct PaintedTile {
    texture: Texture2D,
    tile_id: String,
}

impl Map {
    /// Creates a map with the provided pixel dimensions and tile size.
    pub fn new(map_dimension: Size, tile_size: Size) -> Self {
        let map_width_tiles: usize = dimension_to_tiles(map_dimension.width);
        let map_height_tiles: usize = dimension_to_tiles(map_dimension.height);
        let tiles: Vec<Option<PaintedTile>> = vec![None; map_width_tiles * map_height_tiles];

        let grid_size: Vec2 = vec2(
            map_width_tiles as f32 * tile_size.width,
            map_height_tiles as f32 * tile_size.height,
        );

        let camera_center: AxisPosition = grid_size.into();

        Self {
            camera_controller: CameraController::new(camera_center),
            tile_dimensions: tile_size,
            map_width_tiles,
            map_height_tiles,
            tiles,
        }
    }

    /// Returns an immutable reference to the camera controller.
    pub fn get_camera_controller(&self) -> &CameraController {
        &self.camera_controller
    }

    /// Returns a mutable reference to the camera controller.
    pub fn get_camera_controller_mut(&mut self) -> &mut CameraController {
        &mut self.camera_controller
    }

    /// Draws the map contents and returns the active camera used for the draw call.
    pub fn draw(&mut self) -> Camera2D {
        let grid_size = self.grid_size();

        self.camera_controller.update(grid_size);

        let camera = self.camera_controller.to_camera2d();
        set_camera(&camera);

        self.draw_tiles();
        self.setup_grid();
        self.highlight_hovered_tile(&camera);

        set_default_camera();
        camera
    }

    fn setup_grid(&self) {
        let Size { width: tile_width, height: tile_height } = self.tile_dimensions;

        let grid_width = self.map_width_tiles as f32 * tile_width;
        let grid_height = self.map_height_tiles as f32 * tile_height;
        let thickness = 0.0;

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

    /// Returns the `(x, y)` tile coordinates currently under the mouse cursor.
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

    /// Paints a tile slot with the sprite, replacing any previous texture.
    pub fn paint_tile(&mut self, tile_x: usize, tile_y: usize, sprite: &TileSprite) {
        if let Some(index) = self.tile_index(tile_x, tile_y) {
            self.tiles[index] = Some(PaintedTile {
                texture: sprite.texture.clone(),
                tile_id: sprite.id.clone(),
            });
        }
    }

    fn tile_index(&self, tile_x: usize, tile_y: usize) -> Option<usize> {
        if tile_x >= self.map_width_tiles || tile_y >= self.map_height_tiles {
            return None;
        }

        Some(tile_y * self.map_width_tiles + tile_x)
    }

    fn grid_size(&self) -> Vec2 {
        vec2(
            self.map_width_tiles as f32 * self.tile_dimensions.width,
            self.map_height_tiles as f32 * self.tile_dimensions.height,
        )
    }

    /// Writes the current map state to disk in JSON format.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), io::Error> {
        let export = self.export();
        let json = serde_json::to_string_pretty(&export)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        fs::write(path, json)
    }

    /// Loads map data from disk and rebuilds the internal tile buffers.
    pub fn load_from_file<P: AsRef<Path>>(
        &mut self, path: P, catalog: &AssetCatalog,
    ) -> Result<(), MapLoadError> {
        let data = fs::read_to_string(path)?;
        let export: MapExport = serde_json::from_str(&data)?;

        let expected_tile_count = export.width * export.height;
        if export.tiles.len() != expected_tile_count {
            return Err(MapLoadError::TileCountMismatch {
                expected: expected_tile_count,
                found: export.tiles.len(),
            });
        }

        self.map_width_tiles = export.width;
        self.map_height_tiles = export.height;
        self.tile_dimensions = Size {
            width: export.tile_width,
            height: export.tile_height,
        };

        self.tiles = export
            .tiles
            .into_iter()
            .map(|maybe_id| match maybe_id {
                Some(id) => {
                    let sprite = catalog
                        .sprite_by_id(&id)
                        .ok_or_else(|| MapLoadError::UnknownTile(id.clone()))?;
                    Ok(Some(PaintedTile {
                        texture: sprite.texture.clone(),
                        tile_id: sprite.id.clone(),
                    }))
                }
                None => Ok(None),
            })
            .collect::<Result<Vec<_>, MapLoadError>>()?;

        self.camera_controller.screen_center = self.grid_size().into();

        Ok(())
    }

    fn export(&self) -> MapExport {
        let tiles = self
            .tiles
            .iter()
            .map(|tile| {
                tile.as_ref()
                    .map(|painted| painted.tile_id.clone())
            })
            .collect();

        MapExport {
            width: self.map_width_tiles,
            height: self.map_height_tiles,
            tile_width: self.tile_dimensions.width,
            tile_height: self.tile_dimensions.height,
            tiles,
        }
    }
}

/// Converts a raw dimension into an integral number of tiles.
fn dimension_to_tiles(value: f32) -> usize {
    value.max(1.0).round() as usize
}

/// Possible failures when loading a map from disk.
#[derive(Debug)]
pub enum MapLoadError {
    Io(io::Error),
    Parse(serde_json::Error),
    TileCountMismatch {
        expected: usize,
        found: usize,
    },
    UnknownTile(String),
}

impl From<io::Error> for MapLoadError {
    fn from(value: io::Error) -> Self {
        MapLoadError::Io(value)
    }
}

impl From<serde_json::Error> for MapLoadError {
    fn from(value: serde_json::Error) -> Self {
        MapLoadError::Parse(value)
    }
}

impl std::fmt::Display for MapLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapLoadError::Io(err) => write!(f, "IO error: {err}"),
            MapLoadError::Parse(err) => write!(f, "JSON parse error: {err}"),
            MapLoadError::TileCountMismatch { expected, found } => {
                write!(f, "Tile count mismatch. Expected {expected}, found {found}")
            }
            MapLoadError::UnknownTile(id) => write!(f, "Unknown tile id: {id}"),
        }
    }
}

impl std::error::Error for MapLoadError {}

#[derive(Serialize, Deserialize)]
struct MapExport {
    width: usize,
    height: usize,
    tile_width: f32,
    tile_height: f32,
    tiles: Vec<Option<String>>,
}
