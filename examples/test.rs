use diode::graphics::*;
use diode::app::*;
use diode::app::AppConfig;
use lyon::path::Path;
use lyon::math::point;

struct TestApp {
}

impl App for TestApp {
    fn update(&mut self, ctx: &mut Context) {
        ctx.gfx.fill_rect(50.0, 70.0, 200.0, 300.0,
                          &Color::rgba(1.0, 0.0, 0.0, 0.5));
        ctx.gfx.draw_rect(50.0, 70.0, 200.0, 300.0,
                          &Color::rgba(1.0, 1.0, 1.0, 0.1));
        ctx.gfx.draw_line(50.0, 70.0, 200.0, 300.0,
                          &Color::rgba(1.0, 1.0, 1.0, 0.1));
        ctx.gfx.fill_rect(120.0, 120.0, 400.0, 400.0,
                          &Color::rgba(0.0, 1.0, 0.0, 0.5));
        /*
        for i in 0..100 {
            ctx.gfx.draw_text("hello world", 40.0, 100.0, 100.0 + i as f32, 1000.0,
                              &Color::rgb(i as f32 / 100.0, 0.0, 1.0));
        }
        */
        let mut builder = Path::builder();
        builder.begin(point(100.0, 100.0));
        builder.line_to(point(200.0, 200.0));
        builder.line_to(point(300.0, 400.0));
        builder.close();
        let path = builder.build();
        ctx.gfx.fill_path(path, &Color::rgba(0.0, 1.0, 1.0, 1.0));

        let mut builder = Path::builder();
        builder.begin(point(100.0, 100.0));
        builder.line_to(point(200.0, 200.0));
        builder.line_to(point(300.0, 400.0));
        builder.close();
        let path = builder.build();
        ctx.gfx.draw_path(path, &Color::rgba(0.0, 0.0, 0.0, 1.0));

        ctx.gfx.draw_fitted_text_line("the quick brown fox jumps over the lazy dog",
                                      12.0, 100.0, 420.0, ctx.gfx.width - 100.0,
                                      &Color::rgb(1.0, 1.0, 1.0));
        ctx.gfx.draw_text("the quick brown fox jumps over the lazy dog",
                          12.0, 100.0, 500.0, Some(ctx.gfx.width - 100.0), None,
                          &Color::rgb(1.0, 1.0, 1.0));
    }
}

fn main() {
    let cfg = AppConfig {
        title: "renderer test".into(),
        width: 800,
        height: 600,
        ..Default::default()
    };
    let app = TestApp {};
    run(cfg, app);
}
