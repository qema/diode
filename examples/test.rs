use diode::graphics::*;
use diode::app::*;
use lyon::path::Path;
use lyon::math::point;

fn main() {
    run(move |ctx| {
        ctx.renderer.fill_rect(&mut ctx.gfx,
                               50.0, 70.0, 200.0, 300.0,
                               &Color::rgba(1.0, 0.0, 0.0, 0.5));
        ctx.renderer.fill_rect(&mut ctx.gfx,
                               120.0, 120.0, 400.0, 400.0,
                               &Color::rgba(0.0, 1.0, 0.0, 0.5));
        for i in 0..100 {
            ctx.renderer.draw_text(&mut ctx.gfx, "hello world", 40.0, 100.0, 100.0 + i as f32,
                                   &Color::rgb(i as f32 / 100.0, 0.0, 1.0));
        }
        let mut builder = Path::builder();
        builder.begin(point(100.0, 100.0));
        builder.line_to(point(200.0, 200.0));
        builder.line_to(point(300.0, 400.0));
        builder.close();
        let path = builder.build();
        ctx.renderer.fill_path(&mut ctx.gfx, path, &Color::rgba(0.0, 1.0, 1.0, 1.0));

        let mut builder = Path::builder();
        builder.begin(point(100.0, 100.0));
        builder.line_to(point(200.0, 200.0));
        builder.line_to(point(300.0, 400.0));
        builder.close();
        let path = builder.build();
        ctx.renderer.stroke_path(&mut ctx.gfx, path, &Color::rgba(0.0, 0.0, 0.0, 1.0));

        ctx.renderer.draw_text(&mut ctx.gfx, "the quick brown fox jumps over the lazy dog",
                               12.0, 100.0, 500.0,
                               &Color::rgb(1.0, 1.0, 1.0));
    });
}
