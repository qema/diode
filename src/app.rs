use std::{
    thread::sleep,
    time::{Duration, SystemTime},
};

use crate::graphics::*;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub struct AppSettings {
    pub title: String,
    pub width: u32,
    pub height: u32,
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            title: "App".into(),
            width: 800,
            height: 600,
        }
    }
}

pub trait App {
    fn init(&mut self, _gfx: &mut Graphics) {}
    fn update(&mut self, _gfx: &mut Graphics) {}
    fn handle_event(&mut self, _gfx: &mut Graphics, _event: &WindowEvent) {}
}

async fn run_async(event_loop: EventLoop<()>, window: &Window, mut app: impl App) {
    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.height.max(1);

    let instance = wgpu::Instance::default();

    let surface = instance.create_surface(&window).unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .unwrap();
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

    let mut gfx = Graphics::init(config, device, queue);
    gfx.resize(
        gfx.config.width,
        gfx.config.height,
        window.scale_factor() as f32,
    );

    // add blank texture at uv coords (0, 0)
    gfx.add_texture(&[0xff; 4], 1, 1);

    app.init(&mut gfx);

    event_loop
        .run(move |event, target| {
            let _ = (&instance, &adapter);

            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                app.handle_event(&mut gfx, &event);

                match event {
                    WindowEvent::Resized(new_size) => {
                        gfx.resize(
                            new_size.width,
                            new_size.height,
                            window.scale_factor() as f32,
                        );
                        surface.configure(&gfx.device, &gfx.config);
                        window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        let frame = surface.get_current_texture().unwrap();
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        gfx.render(&view);
                        frame.present();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {
                        app.update(&mut gfx);
                        gfx.commit_geom();
                        window.request_redraw();
                    }
                }
            }

            //sleep(Duration::from_millis(16));
        })
        .unwrap();
}

pub fn run(settings: AppSettings, app: impl App) {
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    window.set_title(&settings.title);
    let _ = window.request_inner_size(LogicalSize::new(settings.width, settings.height));
    pollster::block_on(run_async(event_loop, &window, app));
}
