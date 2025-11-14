use crate::core::assets::{AssetCatalog, AssetCategory, TileSprite};
use crate::core::map::map::{Map, MapLoadError};
use crate::core::map::tile::Size;
use image::imageops::FilterType;
use macroquad::color::{BLACK, DARKGRAY, WHITE};
use macroquad::input::{MouseButton, is_mouse_button_down, mouse_position};
use macroquad::math::{Rect, Vec2, vec2};
use macroquad::miniquad::conf::Icon;
use macroquad::prelude::{Camera2D, clear_background};
use macroquad::text::draw_text;
use macroquad::ui::{Ui, hash, root_ui, widgets};
use macroquad::window::{Conf, next_frame, screen_height};
use std::convert::TryInto;

mod core;

fn window_conf() -> Conf {
    Conf {
        window_title: "ForgeTile".into(),
        fullscreen: false,
        icon: Some(load_app_icon()),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let map_size = Size { width: 20.0, height: 15.0 };
    let tile_size = Size { width: 32.0, height: 32.0 };
    let mut map = Map::new(map_size, tile_size);
    let asset_catalog = AssetCatalog::load(tile_size).await;
    let mut palette_panel = PalettePanel::new(tile_size);

    loop {
        clear_background(BLACK);

        draw_text("ForgeTile!", 20.0, 20.0, 30.0, DARKGRAY);

        let camera: Camera2D = map.draw();
        let zoom = map
            .get_camera_controller()
            .get_current_zoom();

        draw_text(&format!("Zoom: {:.1}", zoom), 10.0, 20.0, 20.0, WHITE);

        let panel_actions: PanelActions = palette_panel.draw(&asset_catalog);

        if is_mouse_button_down(MouseButton::Left) && !palette_panel.pointer_over_ui() {
            if let (Some((tile_x, tile_y)), Some(sprite)) =
                (map.hovered_tile(&camera), palette_panel.selected_sprite(&asset_catalog))
            {
                map.paint_tile(tile_x, tile_y, sprite);
            }
        }

        if panel_actions.save_requested {
            match map.save_to_file("map.json") {
                Ok(_) => println!("map.json saved!"),
                Err(err) => eprintln!("Error saving map: {err}"),
            }
        }
        if panel_actions.load_requested {
            log_map_load_result(map.load_from_file("map.json", &asset_catalog));
        }

        next_frame().await;
    }
}

struct PalettePanel {
    selected_category: usize,
    selected_tile: Option<usize>,
    preview_columns: usize,
    tile_preview_size: f32,
    button_padding: f32,
    grid_origin: Vec2,
    window_position: Vec2,
    pointer_over_ui: bool,
}

impl PalettePanel {
    fn new(tile_size: Size) -> Self {
        Self {
            selected_category: 0,
            selected_tile: None,
            preview_columns: 3,
            tile_preview_size: tile_size.width.max(8.0),
            button_padding: 6.0,
            grid_origin: vec2(10.0, 110.0),
            window_position: vec2(20.0, 80.0),
            pointer_over_ui: false,
        }
    }

    fn draw(&mut self, catalog: &AssetCatalog) -> PanelActions {
        let mut actions = PanelActions::default();
        self.ensure_selection_bounds(catalog);
        let panel_height = (screen_height() - 60.0).max(260.0);
        let panel_size = vec2(280.0, panel_height);
        let position = self.window_position;
        let rect = Rect::new(position.x, position.y, panel_size.x, panel_size.y);

        root_ui().window(hash!("palette_window"), position, panel_size, |ui| {
            ui.label(None, "Tile Palette");

            if catalog.is_empty() {
                ui.separator();
                ui.label(None, "No asset tiles were found.");
                ui.label(None, "Add an `assets` folder next to the executable.");
                return;
            }

            let category_labels: Vec<&str> = catalog
                .categories()
                .iter()
                .map(|category| category.name.as_str())
                .collect();

            ui.combo_box(
                hash!("palette_categories"),
                "Categories",
                &category_labels,
                &mut self.selected_category,
            );
            ui.separator();

            if let Some(category) = catalog.category(self.selected_category) {
                if category.tiles.is_empty() {
                    ui.label(None, "No tiles in this category yet.");
                } else {
                    ui.label(None, "Pick a tile, then left click on the grid to paint.");
                    if let Some(index) = self.selected_tile {
                        if let Some(tile) = category.tiles.get(index) {
                            ui.label(None, &format!("Selected: {}", tile.name));
                        }
                    }
                    self.draw_tile_grid(ui, category);
                }
            }

            ui.separator();
            if ui.button(None, "Salvar mapa (JSON)") {
                actions.save_requested = true;
            }
            if ui.button(None, "Carregar mapa (JSON)") {
                actions.load_requested = true;
            }
        });

        let (mouse_x, mouse_y) = mouse_position();
        self.pointer_over_ui = rect.contains(vec2(mouse_x, mouse_y));
        actions
    }

    fn draw_tile_grid(&mut self, ui: &mut Ui, category: &AssetCategory) {
        let columns = self.preview_columns.max(1);
        let button_edge = self.tile_preview_size + self.button_padding;
        let mut x = self.grid_origin.x;
        let mut y = self.grid_origin.y;

        for (index, tile) in category.tiles.iter().enumerate() {
            let pressed = widgets::Button::new(tile.texture.clone())
                .position(vec2(x, y))
                .size(vec2(button_edge, button_edge))
                .selected(self.selected_tile == Some(index))
                .ui(ui);

            if pressed {
                self.selected_tile = Some(index);
            }

            x += button_edge + self.button_padding;
            if (index + 1) % columns == 0 {
                x = self.grid_origin.x;
                y += button_edge + self.button_padding;
            }
        }
    }

    fn pointer_over_ui(&self) -> bool {
        self.pointer_over_ui
    }

    fn selected_sprite<'a>(&self, catalog: &'a AssetCatalog) -> Option<&'a TileSprite> {
        let category = catalog.category(self.selected_category)?;
        let index = self.selected_tile?;
        category.tiles.get(index)
    }

    fn ensure_selection_bounds(&mut self, catalog: &AssetCatalog) {
        let category_count = catalog.categories().len();
        if category_count == 0 {
            self.selected_category = 0;
            self.selected_tile = None;
            return;
        }

        if self.selected_category >= category_count {
            self.selected_category = 0;
            self.selected_tile = None;
        }

        if let Some(category) = catalog.category(self.selected_category) {
            if let Some(index) = self.selected_tile {
                if index >= category.tiles.len() {
                    self.selected_tile = None;
                }
            }
        } else {
            self.selected_tile = None;
        }
    }
}

#[derive(Default)]
struct PanelActions {
    save_requested: bool,
    load_requested: bool,
}

fn log_map_load_result(result: Result<(), MapLoadError>) {
    match result {
        Ok(_) => println!("Mapa carregado de map.json"),
        Err(err) => eprintln!("Erro ao carregar mapa: {err}"),
    }
}

fn load_app_icon() -> Icon {
    const LOGO_BYTES: &[u8] = include_bytes!("../docs/logo.png");
    match image::load_from_memory(LOGO_BYTES) {
        Ok(dynamic) => {
            let rgba = dynamic.to_rgba8();
            Icon {
                small: resize_icon::<16, { 16 * 16 * 4 }>(&rgba),
                medium: resize_icon::<32, { 32 * 32 * 4 }>(&rgba),
                big: resize_icon::<64, { 64 * 64 * 4 }>(&rgba),
            }
        }
        Err(err) => {
            eprintln!("Failed to load window icon, falling back to default: {err}");
            Icon::miniquad_logo()
        }
    }
}

fn resize_icon<const SIZE: u32, const LEN: usize>(image: &image::RgbaImage) -> [u8; LEN] {
    let resized = image::imageops::resize(image, SIZE, SIZE, FilterType::CatmullRom);
    resized
        .into_raw()
        .try_into()
        .unwrap_or_else(|_| panic!("Unexpected icon buffer length for {SIZE}px icon"))
}
