mod batch;
mod pass;
mod primitive;
pub mod shadow;
mod text;

use nalgebra as na;
use web_sys::HtmlCanvasElement;

use crate::{geom::Screen, Error};

pub use batch::{Batch, DrawUnit, LineBatch, TriBatch};
pub use pass::{ColPass, TexColPass};
pub use primitive::{
    ColVertex, Geometry, GeometryMode, Line, Quad, TexColVertex, TexVertex, Triangle, Vertex,
};
pub use shadow::{Light, OccluderBatch, ShadowMap, ShadowedColorPass};
pub use text::{Font, TextBatch};

pub struct Draw {
    canvas: HtmlCanvasElement,
    golem_ctx: golem::Context,
}

impl Draw {
    pub fn new(canvas: HtmlCanvasElement, golem_ctx: golem::Context) -> Result<Self, Error> {
        match_hidpi_canvas_size(&canvas);

        Ok(Draw { canvas, golem_ctx })
    }

    pub fn screen(&self) -> Screen {
        Screen {
            size: na::Vector2::new(self.canvas.width(), self.canvas.height()),
            device_pixel_ratio: device_pixel_ratio(),
        }
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
