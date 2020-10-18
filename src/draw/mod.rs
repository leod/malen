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
        Ok(Draw { canvas, golem_ctx })
    }

    pub fn screen(&self) -> Screen {
        Screen {
            size: na::Vector2::new(self.canvas.width(), self.canvas.height()),
        }
    }

    pub fn golem_ctx(&self) -> &golem::Context {
        &self.golem_ctx
    }
}
