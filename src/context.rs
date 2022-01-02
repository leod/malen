use std::rc::Rc;

use web_sys::HtmlCanvasElement;

use crate::{
    error::InitError,
    geometry::{ColorVertex, SpriteVertex},
    gl::{self, DrawUnit, Element, Texture},
    input::InputState,
    pass::{ColorPass, Matrices, SpritePass},
    Canvas, Color4, Config, DrawParams, Event, Screen, UniformBuffer,
};

pub struct Context {
    canvas: Canvas,
    input_state: InputState,

    sprite_pass: SpritePass,
    color_pass: ColorPass,
}

impl Context {
    pub fn from_canvas_element_id(id: &str, config: Config) -> Result<Self, InitError> {
        Self::from_canvas(
            Canvas::from_element_id(id, config.canvas_size.clone())?,
            config,
        )
    }

    pub fn from_canvas_element(
        html_element: HtmlCanvasElement,
        config: Config,
    ) -> Result<Self, InitError> {
        Self::from_canvas(
            Canvas::from_element(html_element, config.canvas_size.clone())?,
            config,
        )
    }

    fn from_canvas(canvas: Canvas, _: Config) -> Result<Self, InitError> {
        let input_state = InputState::default();
        let sprite_pass = SpritePass::new(canvas.gl())?;
        let color_pass = ColorPass::new(canvas.gl())?;

        Ok(Context {
            canvas,
            input_state,
            sprite_pass,
            color_pass,
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

    pub fn color_pass(&self) -> &ColorPass {
        &self.color_pass
    }

    pub fn screen(&self) -> Screen {
        self.canvas.screen()
    }

    pub fn clear(&self, color: Color4) {
        self.canvas.clear(color);
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        let event = self.canvas.pop_event()?;
        self.input_state.handle_event(&event);
        Some(event)
    }

    pub fn draw_sprites<E>(
        &self,
        matrices: &UniformBuffer<Matrices>,
        texture: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
        params: &DrawParams,
    ) where
        E: Element,
    {
        self.sprite_pass.draw(matrices, texture, draw_unit, params);
    }

    pub fn draw_colors<E>(
        &self,
        matrices: &UniformBuffer<Matrices>,
        draw_unit: DrawUnit<ColorVertex, E>,
        params: &DrawParams,
    ) where
        E: Element,
    {
        self.color_pass.draw(matrices, draw_unit, params);
    }
}
