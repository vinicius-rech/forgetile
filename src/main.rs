use crate::core::assets::{AssetCatalog, AssetCategory, TileSprite};
use crate::core::map::map::Map;
use crate::core::map::tile::Size;
use macroquad::color::{BLACK, DARKGRAY, WHITE};
use macroquad::input::{MouseButton, is_mouse_button_down, mouse_position};
use macroquad::math::{Rect, Vec2, vec2};
use macroquad::prelude::{Camera2D, clear_background};
use macroquad::text::draw_text;
use macroquad::ui::{Ui, hash, root_ui, widgets};
use macroquad::window::{Conf, next_frame, screen_height};

mod core;

fn window_conf() -> Conf {
    Conf {
        window_title: "ForgeTile".into(),
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let map_dimension = Size { width: 100.0, height: 100.0 };
    let tile_size = Size { width: 32.0, height: 32.0 };
    let asset_catalog = AssetCatalog::load(tile_size).await;
    let mut map = Map::new(map_dimension, tile_size);
    let mut palette_panel = PalettePanel::new(tile_size);

    loop {
        clear_background(BLACK);

        draw_text("ForgeTile!", 20.0, 20.0, 30.0, DARKGRAY);

        let camera: Camera2D = map.draw();

        let panel_actions = palette_panel.draw(&asset_catalog);

        if is_mouse_button_down(MouseButton::Left) && !palette_panel.pointer_over_ui() {
            if let (Some((tile_x, tile_y)), Some(sprite)) =
                (map.hovered_tile(&camera), palette_panel.selected_sprite(&asset_catalog))
            {
                map.paint_tile(tile_x, tile_y, sprite);
            }
        }

        if panel_actions.save_requested {
            match map.save_to_file("map.json") {
                Ok(_) => println!("Mapa salvo em map.json"),
                Err(err) => eprintln!("Erro ao salvar mapa: {err}"),
            }
        }
        if panel_actions.load_requested {
            match map.load_from_file("map.json", &asset_catalog) {
                Ok(_) => println!("Mapa carregado de map.json"),
                Err(err) => eprintln!("Erro ao carregar mapa: {err}"),
            }
        }

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
