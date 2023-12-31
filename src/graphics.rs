use bytemuck::{Pod, Zeroable};
use fontdue::layout::{CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle};
use fontdue::{Font, FontSettings};
use lyon::geom::euclid::{Box2D, Point2D};
use lyon::math::point;
use lyon::path::{Path, Winding};
use lyon::tessellation::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::{borrow::Cow, mem};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

#[derive(Debug)]
pub struct Rect {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}
impl Rect {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }
    pub fn zero() -> Self {
        Self {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Color {
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

pub struct Graphics {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    n_committed_indices: u32,
    pub config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    uniform_buf: wgpu::Buffer,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    texture_cur_x: u32,
    texture_cur_y: u32,
    texture_cur_max_height: u32,
    font: Font,
    font_atlas: HashMap<GlyphRasterConfig, Rect>,
}

const MAX_N_VERTICES: usize = 100000;
const MAX_N_INDICES: usize = 100000;
const TEXTURE_SIZE: u32 = 1000;

impl Graphics {
    pub fn init(
        config: wgpu::SurfaceConfiguration,
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Self {
        let vertices = [Vertex {
            pos: [0.0, 0.0],
            uv: [0.0, 0.0],
            color: [0.0, 0.0, 0.0, 0.0],
        }; MAX_N_VERTICES];
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let indices = [0u32; MAX_N_INDICES];
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        let size = [config.width as f32, config.height as f32];
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&size),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(8),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let vertex_bufs = [wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 2,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 4 * 4,
                    shader_location: 2,
                },
            ],
        }];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_bufs,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.view_formats[0].into(),
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let texture_extent = wgpu::Extent3d {
            width: TEXTURE_SIZE,
            height: TEXTURE_SIZE,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        let font = Font::from_bytes(
            include_bytes!("../resources/WorkSans-Light.ttf") as &[u8],
            FontSettings::default(),
        )
        .unwrap();

        Self {
            width: 0.0,
            height: 0.0,
            scale: 0.0,
            indices: vec![],
            vertices: vec![],
            n_committed_indices: 0,
            config,
            device,
            queue,
            vertex_buf,
            index_buf,
            uniform_buf,
            texture,
            bind_group,
            pipeline,
            texture_cur_x: 0,
            texture_cur_y: 0,
            texture_cur_max_height: 0,
            font,
            font_atlas: HashMap::new(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32, scale: f32) {
        let width = width.max(1);
        let height = height.max(1);
        self.width = width as f32 / scale;
        self.height = height as f32 / scale;
        self.scale = scale;
        self.config.width = width;
        self.config.height = height;
        let size = [self.width, self.height];
        self.queue
            .write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&size));
    }

    pub fn add_geom(&mut self, vertices: &[Vertex], indices: &[u32]) {
        self.indices.append(
            &mut indices
                .iter()
                .map(|&idx| idx + self.vertices.len() as u32)
                .collect::<Vec<_>>()
                .to_vec(),
        );
        self.vertices.append(&mut vertices.to_vec());
        self.n_committed_indices = self.indices.len() as u32;
    }

    pub fn commit_geom(&mut self) {
        self.queue
            .write_buffer(&self.vertex_buf, 0, bytemuck::cast_slice(&self.vertices));
        self.queue
            .write_buffer(&self.index_buf, 0, bytemuck::cast_slice(&self.indices));
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn add_texture(&mut self, data: &[u8], width: u32, height: u32) -> Rect {
        if self.texture_cur_x + width >= TEXTURE_SIZE {
            self.texture_cur_x = 0;
            self.texture_cur_y += self.texture_cur_max_height;
            self.texture_cur_max_height = 0;
        }
        if self.texture_cur_y + height >= TEXTURE_SIZE {
            eprintln!("out of texture space");
        }
        let dst_x = self.texture_cur_x;
        let dst_y = self.texture_cur_y;

        self.texture_cur_x += width;
        self.texture_cur_max_height = self.texture_cur_max_height.max(height);

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: dst_x,
                    y: dst_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        Rect {
            x1: dst_x as f32 / TEXTURE_SIZE as f32,
            y1: dst_y as f32 / TEXTURE_SIZE as f32,
            x2: (dst_x + width) as f32 / TEXTURE_SIZE as f32,
            y2: (dst_y + height) as f32 / TEXTURE_SIZE as f32,
        }
    }

    pub fn render(&mut self, view: &wgpu::TextureView) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.pipeline);
            if self.n_committed_indices > 0 {
                rpass.set_bind_group(0, &self.bind_group, &[]);
                rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint32);
                rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
                rpass.draw_indexed(0..self.n_committed_indices, 0, 0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));
    }

    pub fn draw_path(&mut self, path: Path, color: &Color) {
        let mut geometry: VertexBuffers<Vertex, u32> = VertexBuffers::new();
        let mut tessellator = StrokeTessellator::new();
        let color_v = [color.r, color.g, color.b, color.a];
        {
            tessellator
                .tessellate_path(
                    &path,
                    &StrokeOptions::default(),
                    &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| Vertex {
                        pos: vertex.position().to_array(),
                        uv: [0.0, 0.0],
                        color: color_v,
                    }),
                )
                .unwrap();
        }
        self.add_geom(&geometry.vertices, &geometry.indices);
    }

    pub fn fill_path(&mut self, path: Path, color: &Color) {
        let mut geometry: VertexBuffers<Vertex, u32> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();
        let color_v = [color.r, color.g, color.b, color.a];
        {
            tessellator
                .tessellate_path(
                    &path,
                    &FillOptions::default(),
                    &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| Vertex {
                        pos: vertex.position().to_array(),
                        uv: [0.0, 0.0],
                        color: color_v,
                    }),
                )
                .unwrap();
        }
        self.add_geom(&geometry.vertices, &geometry.indices);
    }

    pub fn fill_rect(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: &Color) {
        let color_v = [color.r, color.g, color.b, color.a];
        let vertices = [
            Vertex {
                pos: [x1, y1],
                uv: [0.0, 0.0],
                color: color_v,
            },
            Vertex {
                pos: [x1, y2],
                uv: [0.0, 0.0],
                color: color_v,
            },
            Vertex {
                pos: [x2, y2],
                uv: [0.0, 0.0],
                color: color_v,
            },
            Vertex {
                pos: [x2, y1],
                uv: [0.0, 0.0],
                color: color_v,
            },
        ];
        let indices = [0u32, 1, 2, 0, 2, 3];
        self.add_geom(&vertices, &indices);
    }

    pub fn fill_tri(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        color: &Color,
    ) {
        let color_v = [color.r, color.g, color.b, color.a];
        let vertices = [
            Vertex {
                pos: [x1, y1],
                uv: [0.0, 0.0],
                color: color_v,
            },
            Vertex {
                pos: [x2, y2],
                uv: [0.0, 0.0],
                color: color_v,
            },
            Vertex {
                pos: [x3, y3],
                uv: [0.0, 0.0],
                color: color_v,
            },
        ];
        let indices = [0u32, 1, 2];
        self.add_geom(&vertices, &indices);
    }

    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: &Color) {
        let mut builder = Path::builder();
        builder.begin(point(x1, y1));
        builder.line_to(point(x2, y2));
        builder.close();
        let path = builder.build();
        self.draw_path(path, color);
    }

    pub fn draw_rect(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: &Color) {
        let mut builder = Path::builder();
        builder.add_rectangle(
            &Box2D::new(Point2D::new(x1, y1), Point2D::new(x2, y2)),
            Winding::Positive,
        );
        let path = builder.build();
        self.draw_path(path, color);
    }

    pub fn draw_tri(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        color: &Color,
    ) {
        let mut builder = Path::builder();
        builder.begin(point(x1, y1));
        builder.line_to(point(x2, y2));
        builder.line_to(point(x3, y3));
        builder.line_to(point(x1, y1));
        builder.close();
        let path = builder.build();
        self.draw_path(path, color);
    }

    pub fn draw_fitted_text_line(
        &mut self,
        text: &str,
        size: f32,
        x: f32,
        y: f32,
        max_width: f32,
        color: &Color,
    ) {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.append(&[&self.font], &TextStyle::new(text, size * self.scale, 0));
        let mut fitted_text = String::new();
        for glyph in layout.glyphs() {
            if (glyph.x + glyph.width as f32) >= max_width * self.scale {
                for _ in 0..3.min(fitted_text.len()) {
                    fitted_text.pop();
                }
                fitted_text.push_str("...");
                break;
            }
            fitted_text.push(glyph.parent);
        }
        self.draw_text(&fitted_text, size, x, y, None, None, color);
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        size: f32,
        x: f32,
        y: f32,
        max_width: Option<f32>,
        max_height: Option<f32>,
        color: &Color,
    ) {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
            max_width: if let Some(w) = max_width {
                Some(w * self.scale)
            } else {
                None
            },
            max_height: if let Some(h) = max_height {
                Some(h * self.scale)
            } else {
                None
            },
            ..Default::default()
        });
        layout.append(&[&self.font], &TextStyle::new(text, size * self.scale, 0));

        for glyph in layout.glyphs() {
            if let Some(h) = max_height {
                if (glyph.y + glyph.height as f32) / self.scale >= h {
                    break;
                }
            }
            if !self.font_atlas.contains_key(&glyph.key) {
                let (metrics, bitmap) = self.font.rasterize(glyph.parent, size * self.scale);
                let mut tex: Vec<u8> = vec![];
                for &v in &bitmap {
                    tex.push(0xff);
                    tex.push(0xff);
                    tex.push(0xff);
                    tex.push(v);
                }
                let rect = self.add_texture(&tex, metrics.width as u32, metrics.height as u32);
                self.font_atlas.insert(glyph.key, rect);
            }
            let uv_rect = &self.font_atlas[&glyph.key];
            let color_v = [color.r, color.g, color.b, color.a];
            let vertices = [
                Vertex {
                    pos: [x + glyph.x / self.scale, y + glyph.y / self.scale],
                    uv: [uv_rect.x1, uv_rect.y1],
                    color: color_v,
                },
                Vertex {
                    pos: [
                        x + glyph.x / self.scale,
                        y + (glyph.y + glyph.height as f32) / self.scale,
                    ],
                    uv: [uv_rect.x1, uv_rect.y2],
                    color: color_v,
                },
                Vertex {
                    pos: [
                        x + (glyph.x + glyph.width as f32) / self.scale,
                        y + (glyph.y + glyph.height as f32) / self.scale,
                    ],
                    uv: [uv_rect.x2, uv_rect.y2],
                    color: color_v,
                },
                Vertex {
                    pos: [
                        x + (glyph.x + glyph.width as f32) / self.scale,
                        y + glyph.y / self.scale,
                    ],
                    uv: [uv_rect.x2, uv_rect.y1],
                    color: color_v,
                },
            ];
            let indices = [0u32, 1, 2, 0, 2, 3];
            self.add_geom(&vertices, &indices);
        }
    }
}
