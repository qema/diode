use diode::app::AppConfig;
use diode::app;

fn main() {
    let settings = AppConfig {
        title: "hello app".into(),
        width: 800,
        height: 600,
    };
    app::run(settings, move |ctx, ui| {
        ui.window(ctx, "hello window", 300.0, 400.0, 0, |ui| {
        });
    });
}
