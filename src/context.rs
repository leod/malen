use std::{cell::RefCell, rc::Rc};

use nalgebra::{Matrix3, Point2, Vector2};
use web_sys::HtmlCanvasElement;

use crate::{
    al,
    data::{Sprite, SpriteBatch},
    error::InitError,
    geom::{Rect, Screen},
    gl::{self, DrawParams, Texture, Uniform},
    input::InputState,
    pass::{ColorPass, InstancedColorPass, MatricesBlock, SpritePass},
    plot::PlotPass,
    Canvas, Color4, Config, Event, FrameError,
};

pub struct Context {
    canvas: Rc<RefCell<Canvas>>,
    input_state: InputState,
    al: Rc<al::Context>,

    sprite_pass: Rc<SpritePass>,
    color_pass: Rc<ColorPass>,
    instanced_color_pass: Rc<InstancedColorPass>,
    plot_pass: Rc<PlotPass>,

    debug_matrices: Option<Uniform<MatricesBlock>>,
    debug_sprite_batch: Option<SpriteBatch>,
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
        let al = Rc::new(al::Context::new()?);

        let sprite_pass = Rc::new(SpritePass::new(canvas.gl())?);
        let color_pass = Rc::new(ColorPass::new(canvas.gl())?);
        let instanced_color_pass = Rc::new(InstancedColorPass::new(canvas.gl())?);
        let plot_pass = Rc::new(PlotPass::new(color_pass.clone()));

        Ok(Context {
            canvas: Rc::new(RefCell::new(canvas)),
            input_state,
            al,
            sprite_pass,
            color_pass,
            instanced_color_pass,
            plot_pass,
            debug_matrices: None,
            debug_sprite_batch: None,
        })
    }

    pub fn canvas(&self) -> Rc<RefCell<Canvas>> {
        self.canvas.clone()
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        let event = self.canvas.borrow_mut().pop_event()?;
        self.input_state.handle_event(&event);
        Some(event)
    }

    pub fn input_state(&self) -> &InputState {
        &self.input_state
    }

    pub fn gl(&self) -> Rc<gl::Context> {
        self.canvas.borrow().gl()
    }

    pub fn al(&self) -> Rc<al::Context> {
        self.al.clone()
    }

    pub fn logical_size(&self) -> Vector2<f32> {
        self.canvas.borrow().logical_size()
    }

    pub fn physical_size(&self) -> Vector2<u32> {
        self.canvas.borrow().physical_size()
    }

    pub fn screen(&self) -> Screen {
        self.canvas.borrow().screen()
    }

    pub fn clear_color_and_depth(&self, color: Color4, depth: f32) {
        gl::clear_color_and_depth(&*self.gl(), color, depth);
    }

    pub fn clear_color(&self, color: Color4) {
        gl::clear_color(&*self.gl(), color);
    }

    pub fn clear_depth(&self, depth: f32) {
        gl::clear_depth(&*self.gl(), depth);
    }

    pub fn sprite_pass(&self) -> Rc<SpritePass> {
        self.sprite_pass.clone()
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

    pub fn draw_debug_texture(&mut self, rect: Rect, texture: &Texture) -> Result<(), FrameError> {
        if self.debug_matrices.is_none() {
            self.debug_matrices = Some(Uniform::new(self.gl(), MatricesBlock::default())?);
        }
        if self.debug_sprite_batch.is_none() {
            self.debug_sprite_batch = Some(SpriteBatch::new(self.gl())?);
        }
        let matrices = self.debug_matrices.as_mut().unwrap();
        let batch = self.debug_sprite_batch.as_mut().unwrap();

        matrices.set(MatricesBlock {
            projection: self.canvas.borrow().screen().project_logical_to_ndc(),
            view: Matrix3::identity(),
        });

        batch.clear();
        batch.push(Sprite {
            rect,
            depth: 0.0,
            tex_rect: Rect::from_top_left(Point2::origin(), texture.size().cast::<f32>()),
            color: Color4::new(1.0, 1.0, 1.0, 1.0),
        });

        self.sprite_pass
            .draw(matrices, texture, batch.draw_unit(), &DrawParams::default());

        Ok(())
    }
}
