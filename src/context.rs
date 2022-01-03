use std::rc::Rc;

use web_sys::HtmlCanvasElement;

use crate::{
    error::InitError,
    geometry::{ColorVertex, SpriteVertex},
    gl::{self, DrawParams, DrawUnit, Element, Texture, UniformBuffer},
    input::InputState,
    pass::{ColorPass, ColorSpritePass, MatricesBlock, SpritePass},
    Canvas, Color4, Config, Event, Screen,
};

pub struct Context {
    canvas: Canvas,
    input_state: InputState,

    sprite_pass: Rc<SpritePass>,
    color_sprite_pass: Rc<ColorSpritePass>,
    color_pass: Rc<ColorPass>,
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
        let color_sprite_pass = ColorSpritePass::new(canvas.gl())?;
        let color_pass = ColorPass::new(canvas.gl())?;

        Ok(Context {
            canvas,
            input_state,
            sprite_pass: Rc::new(sprite_pass),
            color_sprite_pass: Rc::new(color_sprite_pass),
            color_pass: Rc::new(color_pass),
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

    pub fn sprite_pass(&self) -> Rc<SpritePass> {
        self.sprite_pass.clone()
    }

    pub fn color_sprite_pass(&self) -> Rc<ColorSpritePass> {
        self.color_sprite_pass.clone()
    }

    pub fn color_pass(&self) -> Rc<ColorPass> {
        self.color_pass.clone()
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
        matrices: &UniformBuffer<MatricesBlock>,
        texture: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
        params: &DrawParams,
    ) -> Result<(), gl::Error>
    where
        E: Element,
    {
        self.sprite_pass.draw(matrices, texture, draw_unit, params)
    }

    pub fn draw_colors<E>(
        &self,
        matrices: &UniformBuffer<MatricesBlock>,
        draw_unit: DrawUnit<ColorVertex, E>,
        params: &DrawParams,
    ) where
        E: Element,
    {
        self.color_pass.draw(matrices, draw_unit, params);
    }
}
