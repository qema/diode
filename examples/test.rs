use diode::context::*;
use diode::graphics::*;
use diode::app::*;

struct TestApp {
}

impl App for TestApp {
    fn update(&mut self, ctx: &mut Context) {

        for i in 0..100 {
            ctx.renderer.draw_text(&mut ctx.gfx, "hello world", 40.0, 100.0, 100.0 + i as f32,
                                   Color { r: i as f32 / 100.0, g: 0.0, b: 1.0, a: 1.0 } );
        }
    }
}

fn main() {
    let mut app = TestApp {};
    run(&mut app);
}
