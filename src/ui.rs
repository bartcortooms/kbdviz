use crate::compose::{ComposeEntry, ComposeIndex};
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache};
use smithay_client_toolkit::{
    reexports::client::protocol::{wl_shm, wl_surface::WlSurface},
    shm::{slot::SlotPool, Shm},
};
use std::sync::Arc;
use tiny_skia::{Color, Pixmap};
use xkbcommon::xkb;

fn bg_color() -> Color { Color::from_rgba(0.11, 0.11, 0.13, 1.0).unwrap() }
fn text_primary() -> Color { Color::from_rgba(1.0, 1.0, 1.0, 1.0).unwrap() }
fn text_secondary() -> Color { Color::from_rgba(0.75, 0.75, 0.78, 1.0).unwrap() }  // Lighter for better readability
fn text_tertiary() -> Color { Color::from_rgba(0.5, 0.5, 0.55, 1.0).unwrap() }
fn accent_color() -> Color { Color::from_rgba(0.5, 0.7, 1.0, 1.0).unwrap() }

/// A clickable region with its character
struct ClickRegion {
    y_start: f32,
    y_end: f32,
    character: String,
}

pub struct CharRefUI {
    surface: WlSurface,
    width: u32,
    height: u32,
    pixmap: Pixmap,
    pool: SlotPool,

    font_system: FontSystem,
    swash_cache: SwashCache,

    input_text: String,
    compose_index: Arc<ComposeIndex>,

    // Track clickable regions for the current render
    click_regions: Vec<ClickRegion>,
    // Track which row was just copied (by index, for visual feedback)
    copied_row: Option<usize>,
}

impl CharRefUI {
    pub fn new(
        surface: &WlSurface,
        width: u32,
        height: u32,
        shm: &Shm,
        compose_index: Arc<ComposeIndex>,
    ) -> Self {
        let pixmap = Pixmap::new(width, height).unwrap();
        let pool = SlotPool::new((width * height * 4) as usize, shm)
            .expect("Failed to create slot pool");

        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        Self {
            surface: surface.clone(),
            width,
            height,
            pixmap,
            pool,
            font_system,
            swash_cache,
            input_text: String::new(),
            compose_index,
            click_regions: Vec::new(),
            copied_row: None,
        }
    }

    pub fn handle_key_press(&mut self, _raw_code: u32, keysym: xkb::Keysym) {
        // Clear copied indicator on any key press
        self.copied_row = None;

        // Handle backspace
        if keysym == xkb::Keysym::BackSpace {
            self.input_text.clear();
            return;
        }

        // Convert keysym to char
        let utf32 = xkb::keysym_to_utf32(keysym);
        if let Some(ch) = char::from_u32(utf32) {
            if ch.is_alphabetic() {
                // Replace input with just this character (single-letter filter)
                self.input_text.clear();
                self.input_text.push(ch);
            }
        }
    }

    /// Handle a click at the given position, returns the character if one was clicked
    pub fn handle_click(&mut self, _x: f64, y: f64) -> Option<String> {
        let y = y as f32;

        for (index, region) in self.click_regions.iter().enumerate() {
            if y >= region.y_start && y < region.y_end {
                let character = region.character.clone();
                self.copied_row = Some(index);
                return Some(character);
            }
        }

        None
    }

    pub fn render(&mut self) {
        // Clear background
        self.pixmap.fill(bg_color());

        // Clear click regions from previous render
        self.click_regions.clear();

        // Get results
        let results = if self.input_text.is_empty() {
            Vec::new()
        } else {
            self.compose_index.find_variants(&self.input_text)
        };

        // Render input at top
        let input_y = 18.0;

        // Draw input text or hint
        if self.input_text.is_empty() {
            self.draw_text_colored("Type a letter...", 20.0, input_y, 14.0, text_tertiary());
        } else {
            // Just show the filter letter prominently
            let text = self.input_text.clone();
            self.draw_text_colored(&text, 20.0, input_y, 18.0, accent_color());
        }

        // Render results with spacing adjusted for larger text
        let row_height = 34.0;
        let mut y = 50.0;
        if results.is_empty() && !self.input_text.is_empty() {
            self.draw_text_colored("No special characters found", 20.0, y, 13.0, text_tertiary());
        } else if !results.is_empty() {
            for (index, entry) in results.iter().take(10).enumerate() {
                // Check if this row was just copied
                let is_copied = self.copied_row == Some(index);
                self.draw_result(&entry, 20.0, y, row_height, is_copied);

                // Track clickable region
                self.click_regions.push(ClickRegion {
                    y_start: y,
                    y_end: y + row_height,
                    character: entry.character.clone(),
                });

                y += row_height;
            }
        }

        // Show hints when empty
        if self.input_text.is_empty() {
            let hints_y = (self.height as f32) - 80.0;
            self.draw_text_colored("Find special characters:", 20.0, hints_y, 13.0, text_secondary());
            self.draw_text_colored("Try: a e i o u c n s z l y", 20.0, hints_y + 20.0, 12.0, text_tertiary());
            self.draw_text_colored("ESC to close · click to copy", 20.0, hints_y + 45.0, 12.0, text_tertiary());
        }

        // Copy pixmap to Wayland buffer
        // Use Xrgb8888 (no alpha channel) to prevent compositor from blending with windows behind
        let stride = self.width as i32 * 4;
        let (buffer, canvas) = self.pool
            .create_buffer(
                self.width as i32,
                self.height as i32,
                stride,
                wl_shm::Format::Xrgb8888,
            )
            .expect("Failed to create buffer");

        canvas.copy_from_slice(self.pixmap.data());

        self.surface.attach(Some(buffer.wl_buffer()), 0, 0);
        self.surface.damage_buffer(0, 0, self.width as i32, self.height as i32);
    }

    fn draw_row_highlight(&mut self, x: f32, y: f32, w: f32, h: f32) {
        // Draw a subtle lighter background - just slightly brighter than bg_color
        let highlight = tiny_skia::ColorU8::from_rgba(38, 38, 42, 255).premultiply();

        let x_start = x.max(0.0) as usize;
        let x_end = (x + w).min(self.width as f32) as usize;
        let y_start = y.max(0.0) as usize;
        let y_end = (y + h).min(self.height as f32) as usize;

        let pixels = self.pixmap.pixels_mut();
        for py in y_start..y_end {
            for px in x_start..x_end {
                let idx = py * self.width as usize + px;
                pixels[idx] = highlight;
            }
        }
    }

    fn draw_text_colored(&mut self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        use cosmic_text::Color as CosmicColor;

        // Create a buffer for this text
        let metrics = Metrics::new(size, size * 1.4);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        // Borrow the buffer with the font system
        let mut buffer_ref = buffer.borrow_with(&mut self.font_system);

        // Set buffer size and text
        buffer_ref.set_size(Some(self.width as f32), Some(self.height as f32));
        buffer_ref.set_text(text, Attrs::new(), Shaping::Advanced);

        // Shape the text
        buffer_ref.shape_until_scroll(false);

        // Convert tiny_skia Color to CosmicColor
        let text_color = CosmicColor::rgb(
            (color.red() * 255.0) as u8,
            (color.green() * 255.0) as u8,
            (color.blue() * 255.0) as u8
        );

        // Use buffer.draw() which handles glyph rasterization internally
        buffer_ref.draw(&mut self.swash_cache, text_color, |px: i32, py: i32, w: u32, h: u32, color: CosmicColor| {
            // px, py are pixel positions from cosmic-text, offset them to our desired location
            let pixel_x = (x as i32) + px;
            let pixel_y = (y as i32) + py;

            // Only handle single-pixel draws (w=1, h=1)
            if w != 1 || h != 1 {
                return;
            }

            // Check bounds
            if pixel_x < 0 || pixel_x >= self.width as i32 || pixel_y < 0 || pixel_y >= self.height as i32 {
                return;
            }

            // Draw the pixel
            let idx = (pixel_y * self.width as i32 + pixel_x) as usize;
            self.pixmap.pixels_mut()[idx] =
                tiny_skia::ColorU8::from_rgba(color.r(), color.g(), color.b(), color.a()).premultiply();
        });
    }

    fn draw_result(&mut self, entry: &ComposeEntry, x: f32, y: f32, row_height: f32, is_copied: bool) {
        // Draw subtle highlight background if this row was just copied
        if is_copied {
            self.draw_row_highlight(0.0, y - 2.0, self.width as f32, row_height);
        }

        // Draw character (large and prominent) - 28px
        self.draw_text_colored(&entry.character, x, y, 28.0, text_primary());

        // Parse key sequence - formats:
        // Direct: "AltGr-w" or "Shift-a"
        // Dead key: "AltGr-`  e" (double space separates steps)
        // More readable color for modifiers (brighter than before)
        let modifier_color = Color::from_rgba(0.65, 0.65, 0.7, 1.0).unwrap();

        // Check if this is a dead key sequence (has double space)
        if let Some(space_pos) = entry.key_sequence.find("  ") {
            // Dead key sequence: "AltGr-`  e"
            let first_part = &entry.key_sequence[..space_pos];
            let second_part = entry.key_sequence[space_pos..].trim();

            // Parse first part (e.g., "AltGr-`")
            if let Some(dash_pos) = first_part.rfind('-') {
                let modifier = &first_part[..dash_pos];
                let key1 = &first_part[dash_pos + 1..];

                // Draw modifier (readable size and color)
                self.draw_text_colored(modifier, x + 50.0, y + 8.0, 14.0, modifier_color);
                // Draw first key prominent (pushed right for longer modifiers)
                self.draw_text_colored(key1, x + 150.0, y + 2.0, 22.0, accent_color());
                // Draw second key (the base letter)
                self.draw_text_colored(second_part, x + 185.0, y + 2.0, 22.0, accent_color());
            }
        } else if let Some(dash_pos) = entry.key_sequence.rfind('-') {
            // Direct combination: "AltGr-w"
            let modifier = &entry.key_sequence[..dash_pos];
            let key = &entry.key_sequence[dash_pos + 1..];

            // Draw modifier (readable size and color)
            self.draw_text_colored(modifier, x + 50.0, y + 8.0, 14.0, modifier_color);
            // Draw key prominent (pushed right for longer modifiers)
            self.draw_text_colored(key, x + 150.0, y + 2.0, 22.0, accent_color());
        } else {
            // Fallback: just draw as-is
            self.draw_text_colored(&entry.key_sequence, x + 50.0, y + 7.0, 16.0, text_secondary());
        }
    }
}
