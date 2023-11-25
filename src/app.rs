use crate::graphics::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

async fn run_async(event_loop: EventLoop<()>, window: &Window) {
    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.height.max(1);

    let instance = wgpu::Instance::default();

    let surface = instance.create_surface(&window).unwrap();
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }).await.unwrap();

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None)
        .await.unwrap();
    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![swapchain_format],
    };

    surface.configure(&device, &config);

    let mut graphics = Graphics::init(&config, &device, &queue);
    graphics.resize(
        config.width as f32 / window.scale_factor() as f32,
        config.height as f32 / window.scale_factor() as f32,
        &device, &queue);

    // add blank texture
    graphics.add_texture(&[0xff; 4], 1, 1, &queue);

    event_loop.run(move |event, target| {
        let _ = (&instance, &adapter);
        if let Event::WindowEvent { window_id: _, event } = event {
            match event {
                WindowEvent::Resized(new_size) => {
                    config.width = new_size.width.max(1);
                    config.height = new_size.height.max(1);
                    surface.configure(&device, &config);
                    graphics.resize(
                        config.width as f32 / window.scale_factor() as f32,
                        config.height as f32 / window.scale_factor() as f32,
                        &device, &queue);
                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    let frame = surface.get_current_texture().unwrap();
                    let view = frame.texture.create_view(
                        &wgpu::TextureViewDescriptor::default());
                    let vertices = [
                        Vertex{pos: [50., 50.], color: [1.,1.,1.,1.], uv: [0.,0.]},
                        Vertex{pos: [200., 50.], color: [1.,0.,1.,1.], uv: [0.,0.]},
                        Vertex{pos: [220., 300.], color: [0.,1.,1.,1.], uv: [0.,0.]},
                    ];
                    let indices = [0, 1, 2];
                    graphics.add_geom(&vertices, &indices);
                    for i in 0..100 {
                        let vertices = [
                            Vertex{pos: [400.+i as f32, 400.],
                                color: [1.,0.,0.,1.], uv: [0.,0.]},
                            Vertex{pos: [500., 400.], color: [0.,1.,0.,1.], uv: [0.,0.]},
                            Vertex{pos: [500., 500.], color: [0.,0.,1.,1.], uv: [0.,0.]},
                        ];
                        let indices = [0, 1, 2];
                        graphics.add_geom(&vertices, &indices);
                    }
                    graphics.commit_geom(&queue);
                    graphics.render(&view, &device, &queue);
                    frame.present();
                }
                WindowEvent::CloseRequested => target.exit(),
                _ => {}
            }
        }
    }).unwrap();
}

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();
    pollster::block_on(run_async(event_loop, &window));
}
