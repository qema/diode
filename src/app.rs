use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
    dpi::LogicalSize,
};
use crate::graphics::*;

pub struct Context {
    pub gfx: Graphics,
}

pub struct AppConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "App".into(),
            width: 800,
            height: 600,
        }
    }
}

pub trait App {
    fn init(&mut self, _ctx: &mut Context) {}
    fn update(&mut self, _ctx: &mut Context) {}
    fn handle_event(&mut self, _ctx: &mut Context, _event: &WindowEvent) {}
}

async fn run_async(event_loop: EventLoop<()>, window: &Window, mut app: impl App) {
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
    };
    ctx.gfx.resize(ctx.gfx.config.width, ctx.gfx.config.height,
                   window.scale_factor() as f32);

    // add blank texture at uv coords (0, 0)
    ctx.gfx.add_texture(&[0xff; 4], 1, 1);

    app.init(&mut ctx);

    event_loop.run(move |event, target| {
        let _ = (&instance, &adapter);

        if let Event::WindowEvent { window_id: _, event } = event {
            app.handle_event(&mut ctx, &event);

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

                    app.update(&mut ctx);

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

pub fn run(config: AppConfig, app: impl App) {
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();
    window.set_title(&config.title);
    let _ = window.request_inner_size(LogicalSize::new(config.width, config.height));
    pollster::block_on(run_async(event_loop, &window, app));
}
