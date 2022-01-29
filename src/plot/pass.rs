use std::rc::Rc;

use crate::{
    gl::{Blend, DrawParams, Uniform},
    pass::{ColorPass, MatricesBlock},
    text::Font,
};

use super::PlotBatch;

pub struct PlotPass {
    color_pass: Rc<ColorPass>,
}

impl PlotPass {
    pub fn new(color_pass: Rc<ColorPass>) -> Self {
        Self { color_pass }
    }

    pub fn draw(&self, matrices: &Uniform<MatricesBlock>, font: &Font, batch: &mut PlotBatch) {
        let draw_params = DrawParams {
            blend: Some(Blend::default()),
            ..DrawParams::default()
        };

        /*self.color_pass
            .draw(matrices, batch.triangle_batch.draw_unit(), &draw_params);
        self.color_pass
            .draw(matrices, batch.line_batch.draw_unit(), &draw_params);*/
        font.draw(matrices, &mut batch.text_batch);
    }
}
