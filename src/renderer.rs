use crate::graphics::*;
use std::collections::HashMap;
use fontdue::{Font, FontSettings};
use fontdue::layout::{Layout, CoordinateSystem, TextStyle, GlyphRasterConfig};

pub struct Renderer {
    font: Font,
    font_atlas: HashMap<GlyphRasterConfig, Rect>,
}

impl Renderer {
    pub fn new() -> Self {
        let font = Font::from_bytes(
            include_bytes!("../resources/Poppins-Regular.ttf") as &[u8],
            FontSettings::default()).unwrap();
        Self {
            font,
            font_atlas: HashMap::new(),
        }
    }

    pub fn draw_text(&mut self, gfx: &mut Graphics, text: &str, size: f32, x: f32, y: f32,
                     color: Color) {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.append(&[&self.font], &TextStyle::new(text, size, 0));
        for glyph in layout.glyphs() {
            if !self.font_atlas.contains_key(&glyph.key) {
                let (metrics, bitmap) = self.font.rasterize(glyph.parent, size * gfx.scale);
                let mut tex: Vec<u8> = vec![];
                for &v in &bitmap {
                    tex.push(0xff);
                    tex.push(0xff);
                    tex.push(0xff);
                    tex.push(v);
                }
                let rect = gfx.add_texture(&tex, metrics.width as u32, metrics.height as u32);
                self.font_atlas.insert(glyph.key, rect);
            }
            let uv_rect = &self.font_atlas[&glyph.key];
            let color_v = [color.r, color.g, color.b, color.a];
            let vertices = [
                Vertex {
                    pos: [x + glyph.x, y + glyph.y],
                    uv: [uv_rect.x1, uv_rect.y1],
                    color: color_v
                },
                Vertex {
                    pos: [x + glyph.x, y + glyph.y + glyph.height as f32],
                    uv: [uv_rect.x1, uv_rect.y2],
                    color: color_v
                },
                Vertex {
                    pos: [x + glyph.x + glyph.width as f32, y + glyph.y + glyph.height as f32],
                    uv: [uv_rect.x2, uv_rect.y2],
                    color: color_v
                },
                Vertex {
                    pos: [x + glyph.x + glyph.width as f32, y + glyph.y],
                    uv: [uv_rect.x2, uv_rect.y1],
                    color: color_v
                },
            ];
            let indices = [0u16, 1, 2, 0, 2, 3];
            gfx.add_geom(&vertices, &indices);
        }
    }
}
