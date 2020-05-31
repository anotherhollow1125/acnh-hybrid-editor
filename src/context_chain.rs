use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

pub struct ContextChain<'a> {
    ctx: &'a CanvasRenderingContext2d,
}

impl<'a> ContextChain<'a> {
    pub fn new(ctx: &'a CanvasRenderingContext2d) -> Self {
        ContextChain { ctx }
    }

    pub fn begin_path(self) -> Self {
        self.ctx.begin_path();
        self
    }

    pub fn stroke(self) {
        self.ctx.stroke();
    }

    pub fn move_to(self, x: f64, y: f64) -> Self {
        self.ctx.move_to(x, y);
        self
    }

    pub fn line_to(self, x: f64, y: f64) -> Self {
        self.ctx.line_to(x, y);
        self
    }

    pub fn set_line_width(self, value: f64) -> Self {
        self.ctx.set_line_width(value);
        self
    }

    pub fn set_stroke_style(self, value: &JsValue) -> Self {
        self.ctx.set_stroke_style(value);
        self
    }

    pub fn set_fill_style(self, value: &JsValue) -> Self {
        self.ctx.set_fill_style(value);
        self
    }

    pub fn fill_rect(self, x: f64, y: f64, w: f64, h: f64) -> Self {
        self.ctx.fill_rect(x, y, w, h);
        self
    }

    pub fn set_text_baseline(self, s: &str) -> Self {
        self.ctx.set_text_baseline(s);
        self
    }

    pub fn set_text_align(self, s: &str) -> Self {
        self.ctx.set_text_align(s);
        self
    }

    pub fn set_font(self, s: &str) -> Self {
        self.ctx.set_font(s);
        self
    }

    pub fn fill_text(self, s: &str, x: f64, y: f64) -> Self {
        let _ = self.ctx.fill_text(s, x, y);
        self
    }

    /*
        pub fn execute<F: FnOnce(&CanvasRenderingContext2d)>(self, f: F) -> Self {
            f(self.ctx);
            self
        }
    */
}
