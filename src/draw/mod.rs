mod batch;
mod pass;
mod primitive;
pub mod shadow;
mod text;

use web_sys::HtmlCanvasElement;

use nalgebra as na;

use crate::{Color, Error, Point2, Rect, Screen, Vector2};

pub use golem::Texture;

pub use batch::{Batch, DrawUnit, LineBatch, TriBatch};
pub use pass::{ColPass, TexColPass};
pub use primitive::{
    ColVertex, Geometry, GeometryMode, Line, Quad, TexColVertex, TexVertex, Triangle, Vertex,
};
pub use shadow::{Light, OccluderBatch, ShadowMap, ShadowColPass};
pub use text::{Font, TextBatch};

pub struct Draw {
    canvas: HtmlCanvasElement,
    golem_ctx: golem::Context,

    debug_tex_batch: Option<TriBatch<TexColVertex>>,
    debug_tex_pass: Option<TexColPass>,
}

impl Draw {
    pub fn new(canvas: HtmlCanvasElement, golem_ctx: golem::Context) -> Result<Self, Error> {
        match_hidpi_canvas_size(&canvas);

        Ok(Draw {
            canvas,
            golem_ctx,
            debug_tex_batch: None,
            debug_tex_pass: None,
        })
    }

    pub fn screen(&self) -> Screen {
        Screen {
            size: na::Vector2::new(self.canvas.width(), self.canvas.height()),
            device_pixel_ratio: device_pixel_ratio(),
        }
    }

    pub fn debug_tex(&mut self, pos: Point2, tex: &Texture) -> Result<(), Error> {
        let screen = self.screen();
        let size = Vector2::new(tex.width() as f32, tex.height() as f32);

        // We initialize debug batches and shaders lazily, so that they don't
        // have any impact on startup time when not debugging.
        if self.debug_tex_batch.is_none() {
            self.debug_tex_batch = Some(TriBatch::new_golem(&self.golem_ctx)?);
        }
        if self.debug_tex_pass.is_none() {
            self.debug_tex_pass = Some(TexColPass::new_golem(&self.golem_ctx)?);
        }

        let debug_tex_batch = self.debug_tex_batch.as_mut().unwrap();
        let debug_tex_pass = self.debug_tex_pass.as_mut().unwrap();

        debug_tex_batch.clear();
        debug_tex_batch.push_quad(
            &Rect::from_top_left(pos, size).into(),
            0.0,
            Rect::uv_full(),
            Color::new(0.0, 1.0, 1.0, 1.0),
        );
        debug_tex_pass.draw(
            &screen.orthographic_projection(),
            tex,
            &debug_tex_batch.draw_unit(),
        )?;

        Ok(())
    }

    pub fn golem_ctx(&self) -> &golem::Context {
        &self.golem_ctx
    }
}

fn device_pixel_ratio() -> f64 {
    let window = web_sys::window().expect("Failed to obtain window");
    window.device_pixel_ratio()
}

fn match_hidpi_canvas_size(canvas: &HtmlCanvasElement) {
    let scale_factor = device_pixel_ratio();

    let start_width = canvas.width();
    let start_height = canvas.height();

    canvas.set_width((start_width as f64 * scale_factor).round() as u32);
    canvas.set_height((start_height as f64 * scale_factor).round() as u32);

    set_canvas_style_property(canvas, "width", &format!("{}px", start_width));
    set_canvas_style_property(canvas, "height", &format!("{}px", start_height));
}

fn set_canvas_style_property(canvas: &HtmlCanvasElement, property: &str, value: &str) {
    let style = canvas.style();
    style
        .set_property(property, value)
        .expect(&format!("Failed to set {}", property));
}
