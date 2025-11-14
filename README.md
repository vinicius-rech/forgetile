# ForgeTile Map Editor

ForgeTile is a lightweight 2D tile-map editor powered by [macroquad](https://github.com/not-fl3/macroquad). It reads 32×32 sprites directly from an `assets/` folder located next to the executable, lets you paint tiles on a zoomable grid, and exports/imports maps as JSON.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) toolchain (edition 2024, tested with stable)
- The workspace root must contain an `assets/` directory with your spritesheets. Each spritesheet is sliced into 32×32 tiles and grouped by subfolder name for the palette categories.

## Building & Running

1. **Fetch dependencies**
   ```bash
   cargo fetch
   ```
2. **Compile and run the editor**
   ```bash
   cargo run
   ```
   The window starts maximized (windowed). Place your mouse over the grid to see the highlight and left‑click to paint.
3. **Add assets for testing**
   - Drop PNG/JPG spritesheets into `assets/` (use subfolders to create palette categories).
   - Restart the editor (or rerun `cargo run`) to reload new spritesheets.
4. **Export your map**
   - Open the palette window, click **Salvar mapa (JSON)**, and check `map.json` at the project root.
5. **Import a saved map**
   - Keep the same assets available.
   - Click **Carregar mapa (JSON)** to repaint the grid from the last export.

## Current Functionality

- Tile grid rendering with configurable width/height and 32×32 cells.
- Zoom controls (`+`, `-`, `0`) with live HUD feedback.
- Camera panning using right-mouse drag or WASD.
- Automatic asset discovery from the executable’s `assets/` folder (subfolders become palette categories).
- Palette UI with category dropdown, tile selection previews, and current selection status.
- Painting tiles onto the grid via left-click, respecting tile selection.
- JSON map export/import (`map.json`) preserving tile identities.
- Visual hover highlight for precise placement.

## Notes

- Each tile’s JSON entry stores the canonical file path plus tile index. Keep your assets in place when reloading a saved map.
- The editor currently assumes 32×32 sprites. Adjust `tile_size` in `src/main.rs` if you need a different resolution, and ensure your spritesheets match the expected dimensions.
