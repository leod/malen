use std::rc::Rc;

use web_sys::HtmlCanvasElement;

use crate::{
    error::InitError,
    geometry::SpriteBatch,
    gl,
    input::InputState,
    pass::{Matrices, SpritePass},
    Canvas, DrawParams, UniformBuffer,
};

pub struct Context {
    canvas: Canvas,
    input_state: InputState,

    sprite_pass: SpritePass,
}

impl Context {
    pub fn from_canvas_element_id(id: &str) -> Result<Self, InitError> {
        Self::from_canvas(Canvas::from_element_id(id)?)
    }

    pub fn from_canvas_element(html_element: HtmlCanvasElement) -> Result<Self, InitError> {
        Self::from_canvas(Canvas::from_element(html_element)?)
    }

    pub fn from_canvas(canvas: Canvas) -> Result<Self, InitError> {
        let input_state = InputState::default();
        let sprite_pass = SpritePass::new(canvas.gl().clone()).map_err(InitError::OpenGL)?;

        Ok(Context {
            canvas,
            input_state,
            sprite_pass,
        })
    }

    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    pub fn gl(&self) -> Rc<gl::Context> {
        self.canvas.gl()
    }

    pub fn input_state(&self) -> &InputState {
        &self.input_state
    }

    pub fn sprite_pass(&self) -> &SpritePass {
        &self.sprite_pass
    }

    pub fn draw_sprite_batch(
        &self,
        matrices: &UniformBuffer<Matrices>,
        batch: &mut SpriteBatch,
        params: &DrawParams,
    ) {
        self.sprite_pass.draw(matrices, batch.draw_unit(), params);
    }
}
