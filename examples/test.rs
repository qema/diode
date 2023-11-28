use diode::app::AppSettings;
use diode::app::*;
use diode::graphics::*;
use lyon::math::point;
use lyon::path::Path;

struct TestApp {}

impl App for TestApp {
    fn update(&mut self, gfx: &mut Graphics) {
        gfx.fill_rect(50.0, 70.0, 200.0, 300.0, &Color::rgba(1.0, 0.0, 0.0, 0.5));
        gfx.draw_rect(50.0, 70.0, 200.0, 300.0, &Color::rgba(1.0, 1.0, 1.0, 0.1));
        gfx.draw_line(50.0, 70.0, 200.0, 300.0, &Color::rgba(1.0, 1.0, 1.0, 0.1));
        gfx.fill_rect(120.0, 120.0, 400.0, 400.0, &Color::rgba(0.0, 1.0, 0.0, 0.5));

        for i in 0..100 {
            gfx.draw_text(
                "hello world",
                40.0,
                100.0,
                100.0 + i as f32,
                None,
                None,
                &Color::rgb(i as f32 / 100.0, 0.0, 1.0),
            );
        }

        let mut builder = Path::builder();
        builder.begin(point(100.0, 100.0));
        builder.line_to(point(200.0, 200.0));
        builder.line_to(point(300.0, 400.0));
        builder.close();
        let path = builder.build();
        gfx.fill_path(path, &Color::rgba(0.0, 1.0, 1.0, 1.0));

        let mut builder = Path::builder();
        builder.begin(point(100.0, 100.0));
        builder.line_to(point(200.0, 200.0));
        builder.line_to(point(300.0, 400.0));
        builder.close();
        let path = builder.build();
        gfx.draw_path(path, &Color::rgba(0.0, 0.0, 0.0, 1.0));

        gfx.draw_fitted_text_line(
            "the quick brown fox jumps over the lazy dog",
            12.0,
            100.0,
            420.0,
            gfx.width - 100.0,
            &Color::rgb(1.0, 1.0, 1.0),
        );
        gfx.draw_text(
            "the quick brown fox jumps over the lazy dog",
            12.0,
            100.0,
            500.0,
            Some(gfx.width - 100.0),
            None,
            &Color::rgb(1.0, 1.0, 1.0),
        );
    }
}

fn main() {
    let cfg = AppSettings {
        title: "renderer test".into(),
        width: 800,
        height: 600,
        ..Default::default()
    };
    let app = TestApp {};
    run(cfg, app);
}
