// VR overlay rendering
// Text rendering for VR overlays

use crate::utils::error::{Result, VrctError};
use image::{ImageBuffer, Rgba, RgbaImage};
use rusttype::{point, Font, Scale};
use std::sync::Arc;
use tracing::warn;

/// Text renderer for overlays
pub struct TextRenderer {
    font: Arc<Font<'static>>,
}

impl TextRenderer {
    /// Create a new text renderer
    pub fn new() -> Result<Self> {
        // Try to load font from file system first, fall back to embedded font
        let font = Self::load_font()?;

        Ok(Self {
            font: Arc::new(font),
        })
    }

    /// Load font from file system or embedded data
    fn load_font() -> Result<Font<'static>> {
        // Try to load from file system (for development and customization)
        if let Ok(font_data) = std::fs::read("fonts/NotoSansJP-Regular.ttf") {
            if let Some(font) = Font::try_from_vec(font_data) {
                return Ok(font);
            }
        }

        // Fall back to embedded font data
        // For now, we'll use a simple ASCII font as fallback
        // In production, you would embed a proper font file
        warn!("Could not load external font, using fallback");
        
        // Create a minimal font (this is a placeholder - in production you'd embed a real font)
        // For now, return an error to indicate font loading needs to be set up
        Err(VrctError::Overlay(
            "Font file not found. Please ensure fonts/NotoSansJP-Regular.ttf exists".to_string(),
        ))
    }

    /// Render text to an RGBA image
    pub fn render_text(
        &self,
        text: &str,
        width: u32,
        height: u32,
        font_size: f32,
    ) -> Result<RgbaImage> {
        // Create a blank image with transparent background
        let mut image: RgbaImage = ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 0]));

        // Set up font scale
        let scale = Scale::uniform(font_size);

        // Calculate starting position (centered vertically, left-aligned with padding)
        let padding = 10.0;
        let v_metrics = self.font.v_metrics(scale);
        let line_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

        // Split text into lines
        let lines: Vec<&str> = text.lines().collect();
        let total_height = lines.len() as f32 * line_height;
        let start_y = ((height as f32 - total_height) / 2.0).max(padding);

        // Render each line
        for (i, line) in lines.iter().enumerate() {
            let y = start_y + (i as f32 * line_height);
            self.render_line(line, padding, y, scale, &mut image);
        }

        Ok(image)
    }

    /// Render a single line of text
    fn render_line(
        &self,
        text: &str,
        x: f32,
        y: f32,
        scale: Scale,
        image: &mut RgbaImage,
    ) {
        let v_metrics = self.font.v_metrics(scale);
        let offset = point(x, y + v_metrics.ascent);

        let glyphs: Vec<_> = self.font.layout(text, scale, offset).collect();

        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    let px = gx as i32 + bounding_box.min.x;
                    let py = gy as i32 + bounding_box.min.y;

                    if px >= 0
                        && px < image.width() as i32
                        && py >= 0
                        && py < image.height() as i32
                    {
                        let pixel = image.get_pixel_mut(px as u32, py as u32);
                        // White text with alpha based on glyph coverage
                        let alpha = (v * 255.0) as u8;
                        *pixel = Rgba([255, 255, 255, alpha]);
                    }
                });
            }
        }
    }

    /// Render text with word wrapping
    pub fn render_text_wrapped(
        &self,
        text: &str,
        width: u32,
        height: u32,
        font_size: f32,
    ) -> Result<RgbaImage> {
        // Create a blank image with transparent background
        let mut image: RgbaImage = ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 0]));

        // Set up font scale
        let scale = Scale::uniform(font_size);

        // Calculate line metrics
        let v_metrics = self.font.v_metrics(scale);
        let line_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

        let padding = 10.0;
        let max_width = width as f32 - (padding * 2.0);

        // Wrap text
        let wrapped_lines = self.wrap_text(text, max_width, scale);

        // Calculate starting position
        let total_height = wrapped_lines.len() as f32 * line_height;
        let start_y = ((height as f32 - total_height) / 2.0).max(padding);

        // Render each line
        for (i, line) in wrapped_lines.iter().enumerate() {
            let y = start_y + (i as f32 * line_height);
            self.render_line(line, padding, y, scale, &mut image);
        }

        Ok(image)
    }

    /// Wrap text to fit within a given width
    fn wrap_text(&self, text: &str, max_width: f32, scale: Scale) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0;

        for word in text.split_whitespace() {
            let word_width = self.measure_text(word, scale);
            let space_width = self.measure_text(" ", scale);

            if current_width + word_width > max_width && !current_line.is_empty() {
                // Start a new line
                lines.push(current_line.trim().to_string());
                current_line = word.to_string();
                current_width = word_width;
            } else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                    current_width += space_width;
                }
                current_line.push_str(word);
                current_width += word_width;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line.trim().to_string());
        }

        lines
    }

    /// Measure the width of text
    fn measure_text(&self, text: &str, scale: Scale) -> f32 {
        let glyphs: Vec<_> = self.font.layout(text, scale, point(0.0, 0.0)).collect();
        
        if glyphs.is_empty() {
            return 0.0;
        }

        let last_glyph = &glyphs[glyphs.len() - 1];
        let width = last_glyph.position().x + last_glyph.unpositioned().h_metrics().advance_width;
        
        width
    }
}
