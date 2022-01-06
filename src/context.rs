use std::rc::Rc;

use web_sys::HtmlCanvasElement;

use crate::{
    error::InitError,
    gl,
    input::InputState,
    pass::{ColorPass, ColorSpritePass, InstancedColorPass, SpritePass},
    plot::PlotPass,
    Canvas, Config, Event,
};

pub struct Context {
    canvas: Canvas,
    input_state: InputState,

    sprite_pass: Rc<SpritePass>,
    color_sprite_pass: Rc<ColorSpritePass>,
    color_pass: Rc<ColorPass>,
    instanced_color_pass: Rc<InstancedColorPass>,
    plot_pass: Rc<PlotPass>,
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
        let sprite_pass = Rc::new(SpritePass::new(canvas.gl())?);
        let color_sprite_pass = Rc::new(ColorSpritePass::new(canvas.gl())?);
        let color_pass = Rc::new(ColorPass::new(canvas.gl())?);
        let instanced_color_pass = Rc::new(InstancedColorPass::new(canvas.gl())?);
        let plot_pass = Rc::new(PlotPass::new(color_pass.clone()));

        Ok(Context {
            canvas,
            input_state,
            sprite_pass,
            color_sprite_pass,
            color_pass,
            instanced_color_pass,
            plot_pass,
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

    pub fn instanced_color_pass(&self) -> Rc<InstancedColorPass> {
        self.instanced_color_pass.clone()
    }

    pub fn plot_pass(&self) -> Rc<PlotPass> {
        self.plot_pass.clone()
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        let event = self.canvas.pop_event()?;
        self.input_state.handle_event(&event);
        Some(event)
    }
}
