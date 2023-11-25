use bytemuck::{Pod, Zeroable};
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
}

#[derive(Debug)]
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
    indices: Vec<u16>,
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
}

const MAX_N_VERTICES: usize = 100000;
const MAX_N_INDICES: usize = 100000;
const TEXTURE_SIZE: u32 = 1000;

impl Graphics {
    pub fn init(config: wgpu::SurfaceConfiguration,
                device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let vertices = [Vertex {
            pos: [0.0, 0.0],
            uv: [0.0, 0.0],
            color: [0.0, 0.0, 0.0, 0.0]
        }; MAX_N_VERTICES];
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let indices = [0u16; MAX_N_INDICES];
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

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
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
                        count: None
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
                    }

                ],
            });

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
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
                            operation: wgpu::BlendOperation::Add
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })]
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
            format: wgpu::TextureFormat::Rgba8Unorm,
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
                }
            ],
            label: None,
        });

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
        self.queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&size));
    }

    pub fn add_geom(&mut self, vertices: &[Vertex], indices: &[u16]) {
        self.indices.append(&mut indices.iter().map(|&idx| {
            idx + self.vertices.len() as u16
        }).collect::<Vec<_>>().to_vec());
        self.vertices.append(&mut vertices.to_vec());
        self.n_committed_indices = self.indices.len() as u32;
    }

    pub fn commit_geom(&mut self) {
        self.queue.write_buffer(&self.vertex_buf, 0, bytemuck::cast_slice(&self.vertices));
        // wgpu: indices len must be multiple of 2
        if self.indices.len() % 2 == 1 {
            self.indices.push(0);
        }
        self.queue.write_buffer(&self.index_buf, 0, bytemuck::cast_slice(&self.indices));
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn add_texture(&mut self, data: &[u8], width: u32, height: u32,
                       ) -> Rect {
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

        self.queue.write_texture(wgpu::ImageCopyTexture {
            texture: &self.texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: dst_x,
                y: dst_y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        }, data, wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: None,
        }, wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        });
        Rect {
            x1: dst_x as f32 / TEXTURE_SIZE as f32,
            y1: dst_y as f32 / TEXTURE_SIZE as f32,
            x2: (dst_x + width) as f32 / TEXTURE_SIZE as f32,
            y2: (dst_y + height) as f32 / TEXTURE_SIZE as f32,
        }
    }

    pub fn render(&mut self, view: &wgpu::TextureView) {
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0, g: 0.0, b: 0.0, a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    }
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.pipeline);
            if self.n_committed_indices > 0 {
                rpass.set_bind_group(0, &self.bind_group, &[]);
                rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
                rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
                rpass.draw_indexed(0..self.n_committed_indices, 0, 0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));
    }
}
