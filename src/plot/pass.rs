use std::rc::Rc;

use crate::{
    gl::{Blend, DepthTest, DrawParams, UniformBuffer},
    pass::ColorPass,
    text::Font,
    Context, FrameError, MatricesBlock,
};

use super::PlotBatch;

pub struct PlotPass {
    color_pass: Rc<ColorPass>,
}

impl PlotPass {
    pub fn new(context: &Context) -> Self {
        Self {
            color_pass: context.color_pass(),
        }
    }

    pub fn draw(
        &self,
        matrices: &UniformBuffer<MatricesBlock>,
        font: &Font,
        batch: &mut PlotBatch,
    ) -> Result<(), FrameError> {
        let draw_params = DrawParams {
            depth_test: Some(DepthTest::default()),
            blend: Some(Blend::default()),
            ..DrawParams::default()
        };

        self.color_pass
            .draw(matrices, batch.triangle_batch.draw_unit(), &draw_params);
        self.color_pass
            .draw(matrices, batch.line_batch.draw_unit(), &draw_params);
        font.draw(matrices, &mut batch.text_batch)?;

        Ok(())
    }
}
