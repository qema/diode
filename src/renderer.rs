use crate::graphics::*;
use std::collections::HashMap;
use fontdue::{Font, FontSettings};
use fontdue::layout::{Layout, CoordinateSystem, TextStyle, GlyphRasterConfig};
use lyon::path::Path;
use lyon::tessellation::*;

pub struct Renderer {
    font: Font,
    font_atlas: HashMap<GlyphRasterConfig, Rect>,
}

impl Renderer {
    pub fn new() -> Self {
        let font = Font::from_bytes(
            include_bytes!("../resources/WorkSans-Light.ttf") as &[u8],
            FontSettings::default()).unwrap();
        Self {
            font,
            font_atlas: HashMap::new(),
        }
    }

    pub fn stroke_path(&mut self, gfx: &mut Graphics, path: Path, color: &Color) {
        let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();
        let mut tessellator = StrokeTessellator::new();
        let color_v = [color.r, color.g, color.b, color.a];
        {
            tessellator.tessellate_path(
                &path,
                &StrokeOptions::default(),
                &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                    Vertex {
                        pos: vertex.position().to_array(),
                        uv: [0.0, 0.0],
                        color: color_v,
                    }
                }),
            ).unwrap();
        }
        gfx.add_geom(&geometry.vertices, &geometry.indices);
    }

    pub fn fill_path(&mut self, gfx: &mut Graphics, path: Path, color: &Color) {
        let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();
        let color_v = [color.r, color.g, color.b, color.a];
        {
            tessellator.tessellate_path(
                &path,
                &FillOptions::default(),
                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                    Vertex {
                        pos: vertex.position().to_array(),
                        uv: [0.0, 0.0],
                        color: color_v,
                    }
                }),
            ).unwrap();
        }
        gfx.add_geom(&geometry.vertices, &geometry.indices);
    }

    pub fn fill_rect(&mut self, gfx: &mut Graphics, x1: f32, y1: f32, x2: f32, y2: f32,
                     color: &Color) {
        let color_v = [color.r, color.g, color.b, color.a];
        let vertices = [
            Vertex { pos: [x1, y1], uv: [0.0, 0.0], color: color_v },
            Vertex { pos: [x1, y2], uv: [0.0, 0.0], color: color_v },
            Vertex { pos: [x2, y2], uv: [0.0, 0.0], color: color_v },
            Vertex { pos: [x2, y1], uv: [0.0, 0.0], color: color_v },
        ];
        let indices = [0u16, 1, 2, 0, 2, 3];
        gfx.add_geom(&vertices, &indices);
    }

    pub fn draw_text(&mut self, gfx: &mut Graphics, text: &str, size: f32, x: f32, y: f32,
                     color: &Color) {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.append(&[&self.font], &TextStyle::new(text, size * gfx.scale, 0));
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
                    pos: [x + glyph.x/gfx.scale, y + glyph.y/gfx.scale],
                    uv: [uv_rect.x1, uv_rect.y1],
                    color: color_v
                },
                Vertex {
                    pos: [x + glyph.x/gfx.scale, y + (glyph.y+glyph.height as f32)/gfx.scale],
                    uv: [uv_rect.x1, uv_rect.y2],
                    color: color_v
                },
                Vertex {
                    pos: [
                        x + (glyph.x+glyph.width as f32)/gfx.scale,
                        y + (glyph.y+glyph.height as f32)/gfx.scale,
                    ],
                    uv: [uv_rect.x2, uv_rect.y2],
                    color: color_v
                },
                Vertex {
                    pos: [x + (glyph.x+glyph.width as f32)/gfx.scale, y + glyph.y/gfx.scale],
                    uv: [uv_rect.x2, uv_rect.y1],
                    color: color_v
                },
            ];
            let indices = [0u16, 1, 2, 0, 2, 3];
            gfx.add_geom(&vertices, &indices);
        }
    }
}
