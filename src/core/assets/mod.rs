use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use macroquad::math::Rect;
use macroquad::texture::FilterMode;
use macroquad::texture::{Texture2D, load_image};

use crate::core::map::tile::Size;

#[derive(Clone)]
pub struct TileSprite {
    pub name: String,
    pub texture: Texture2D,
}

pub struct AssetCategory {
    pub name: String,
    pub tiles: Vec<TileSprite>,
}

pub struct AssetCatalog {
    categories: Vec<AssetCategory>,
}

impl AssetCatalog {
    pub async fn load(tile_size: Size) -> Self {
        let mut categories = Vec::new();
        if let Some(root) = resolve_assets_root() {
            if let Some(mut root_files) = load_category_from_path(&root, tile_size).await {
                root_files.name = "General".to_string();
                if !root_files.tiles.is_empty() {
                    categories.push(root_files);
                }
            }

            if let Ok(entries) = fs::read_dir(&root) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Some(category) = load_named_category(&entry.path(), tile_size).await
                        {
                            categories.push(category);
                        }
                    }
                }
            }
        } else {
            eprintln!(
                "[assets] Unable to locate an assets directory near the executable. \
                 Place your assets next to the final binary in an `assets` folder."
            );
        }

        Self { categories }
    }

    pub fn categories(&self) -> &[AssetCategory] {
        &self.categories
    }

    pub fn category(&self, index: usize) -> Option<&AssetCategory> {
        self.categories.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.categories.is_empty()
    }
}

impl AssetCategory {
    pub fn new(name: impl Into<String>, tiles: Vec<TileSprite>) -> Self {
        Self { name: name.into(), tiles }
    }
}

async fn load_named_category(path: &Path, tile_size: Size) -> Option<AssetCategory> {
    let tiles = load_tiles_from_directory(path, tile_size).await;
    if tiles.is_empty() {
        return None;
    }

    let name = path
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap_or(Cow::Borrowed("Assets"));
    Some(AssetCategory::new(name.into_owned(), tiles))
}

async fn load_category_from_path(path: &Path, tile_size: Size) -> Option<AssetCategory> {
    if !path.is_dir() {
        return None;
    }

    let tiles = load_tiles_from_directory(path, tile_size).await;
    Some(AssetCategory::new(
        path.file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Assets".to_string()),
        tiles,
    ))
}

async fn load_tiles_from_directory(path: &Path, tile_size: Size) -> Vec<TileSprite> {
    let mut tiles = Vec::new();

    let Ok(entries) = fs::read_dir(path) else {
        eprintln!("[assets] Failed to read directory {:?}", path);
        return tiles;
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_file() && is_supported_image(&entry_path) {
            match load_tiles_from_image(&entry_path, tile_size).await {
                Some(mut sprite_tiles) => tiles.append(&mut sprite_tiles),
                None => {
                    eprintln!("[assets] Could not process {:?}", entry_path);
                }
            }
        }
    }

    tiles
}

fn is_supported_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase()),
        Some(ref ext) if ["png", "jpg", "jpeg"].contains(&ext.as_str())
    )
}

async fn load_tiles_from_image(path: &Path, tile_size: Size) -> Option<Vec<TileSprite>> {
    let image = load_image(path.to_str()?).await.ok()?;
    let (tile_width, tile_height) = size_to_pixels(tile_size)?;

    let columns = image.width() / tile_width;
    let rows = image.height() / tile_height;
    if columns == 0 || rows == 0 {
        return None;
    }

    let file_stem = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let mut sprites = Vec::with_capacity(columns * rows);

    for row in 0..rows {
        for col in 0..columns {
            let rect = Rect::new(
                (col * tile_width) as f32,
                (row * tile_height) as f32,
                tile_width as f32,
                tile_height as f32,
            );
            let tile_image = image.sub_image(rect);
            let texture = Texture2D::from_image(&tile_image);
            texture.set_filter(FilterMode::Nearest);

            let label = format!("{}_{:02}", file_stem, row * columns + col);
            sprites.push(TileSprite { name: label, texture });
        }
    }

    Some(sprites)
}

fn size_to_pixels(size: Size) -> Option<(usize, usize)> {
    let width = size.width.round() as usize;
    let height = size.height.round() as usize;
    if width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

fn resolve_assets_root() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            candidates.push(dir.join("assets"));
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("assets"));
    }

    candidates
        .into_iter()
        .find(|path| path.exists() && path.is_dir())
}
