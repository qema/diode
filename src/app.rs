use crate::context::*;
use crate::renderer::*;
use crate::graphics::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

pub trait App {
    fn update(&mut self, ctx: &mut Context);
}

async fn run_async(
    event_loop: EventLoop<()>, window: &Window, handler: &mut impl App) {
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

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![swapchain_format],
    };

    surface.configure(&device, &config);

    let mut ctx = Context {
        gfx: Graphics::init(config, device, queue),
        renderer: Renderer::new(),
    };
    ctx.gfx.resize(ctx.gfx.config.width, ctx.gfx.config.height,
                   window.scale_factor() as f32);

    // add blank texture
    ctx.gfx.add_texture(&[0xff; 4], 1, 1);

    event_loop.run(move |event, target| {
        let _ = (&instance, &adapter);
        if let Event::WindowEvent { window_id: _, event } = event {
            match event {
                WindowEvent::Resized(new_size) => {
                    ctx.gfx.resize(new_size.width, new_size.height,
                                   window.scale_factor() as f32);
                    surface.configure(&ctx.gfx.device, &ctx.gfx.config);
        
                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    let frame = surface.get_current_texture().unwrap();
                    let view = frame.texture.create_view(
                        &wgpu::TextureViewDescriptor::default());
                    handler.update(&mut ctx);

                    ctx.gfx.commit_geom();
                    ctx.gfx.render(&view);
                    frame.present();
                }
                WindowEvent::CloseRequested => target.exit(),
                _ => {}
            }
        }
    }).unwrap();
}

pub fn run(handler: &mut impl App) {
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();
    pollster::block_on(run_async(event_loop, &window, handler));
}
