use diode::context::*;
use diode::graphics::*;
use diode::app::*;

struct TestApp {
}

impl App for TestApp {
    fn update(&mut self, ctx: &mut Context) {
        /*
        let vertices = [
            Vertex{pos: [50., 50.], color: [1.,1.,1.,1.], uv: [0.,0.]},
            Vertex{pos: [200., 50.], color: [1.,0.,1.,1.], uv: [0.,0.]},
            Vertex{pos: [220., 300.], color: [0.,1.,1.,1.], uv: [0.,0.]},
        ];
        let indices = [0, 1, 2];
        ctx.gfx.add_geom(&vertices, &indices);
        for i in 0..100 {
            let vertices = [
                Vertex{pos: [400.+i as f32, 400.],
                color: [1.,0.,0.,1.], uv: [0.,0.]},
                Vertex{pos: [500., 400.], color: [0.,1.,0.,1.], uv: [0.,0.]},
                Vertex{pos: [500., 500.], color: [0.,0.,1.,1.], uv: [0.,0.]},
            ];
                let indices = [0, 1, 2];
                ctx.gfx.add_geom(&vertices, &indices);
        }
        */
        ctx.renderer.draw_text(&mut ctx.gfx, "hello world", 40.0, 100.0, 100.0);
    }
}

fn main() {
    let mut app = TestApp {};
    run(&mut app);
}
