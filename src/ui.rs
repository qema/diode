use crate::app::Context;
use crate::graphics::*;
use std::collections::HashMap;

struct Window {
    bounds: Rect,
    title: String,
}

pub struct UI {
    windows: HashMap<usize, Window>,
}

impl UI {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }

    pub fn window<F>(&mut self, ctx: &Context, title: &str, width: f32, height: f32,
                     id: usize, update_fn: F)
        where F: Fn(&mut UI) {
        if !self.windows.contains_key(&id) {
            self.windows.insert(id, Window {
                bounds: Rect::new(100.0, 100.0, width, height),
                title: title.to_string()
            });
        }
        update_fn(self);
    }

    pub fn redraw(&mut self, ctx: &Context) {
        for (id, window) in &self.windows {
        }
    }
}
