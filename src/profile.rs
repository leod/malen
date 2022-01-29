use std::{cell::RefCell, collections::VecDeque, rc::Rc, time::Duration};

use instant::Instant;
use nalgebra::{Matrix3, Point2, Vector2};

use crate::{
    data::ColorRect,
    geom::{Rect, Screen},
    gl::{DrawTimer, Uniform},
    pass::MatricesBlock,
    plot::{Axis, LineGraph, Plot, PlotBatch, PlotPass, PlotStyle},
    text::{Font, Text},
    Color4, Context, FrameError, InitError,
};

type FrameTimes = Rc<RefCell<VecDeque<(Instant, Duration)>>>;
type DrawTimes = Rc<RefCell<DrawTimer>>;

#[derive(Debug, Clone)]
pub struct ProfileParams {
    pub margin: Vector2<f32>,
    pub padding: Vector2<f32>,
    pub text_size: f32,
    pub plot_duration: Duration,
    pub plot_size: Vector2<f32>,
    pub plot_style: PlotStyle,
}

impl Default for ProfileParams {
    fn default() -> Self {
        Self {
            margin: Vector2::new(10.0, 10.0),
            padding: Vector2::new(15.0, 15.0),
            text_size: 17.0,
            plot_duration: Duration::from_secs(5),
            plot_size: Vector2::new(700.0, 200.0),
            plot_style: PlotStyle::default(),
        }
    }
}

pub struct Profile {
    font: Font,
    params: ProfileParams,

    screen_matrices: Uniform<MatricesBlock>,
    batch: PlotBatch,
    pass: Rc<PlotPass>,

    dts: VecDeque<(Instant, Duration)>,
    frame_times: FrameTimes,
    draw_times: DrawTimes,
}

pub struct FrameGuard {
    start_time: Instant,
    _profile_guard: coarse_prof::Guard,
    frame_times: FrameTimes,
    draw_times: DrawTimes,
}

impl Profile {
    pub fn new(context: &Context, font: Font, params: ProfileParams) -> Result<Self, InitError> {
        let screen_matrices = Uniform::new(context.gl(), MatricesBlock::default())?;
        let batch = PlotBatch::new(context.gl())?;
        let pass = context.plot_pass();

        let dts = VecDeque::new();
        let frame_times = Rc::new(RefCell::new(VecDeque::new()));
        let draw_times = Rc::new(RefCell::new(DrawTimer::new(
            context.gl(),
            params.plot_duration,
        )));

        Ok(Self {
            font,
            params,
            screen_matrices,
            batch,
            pass,
            dts,
            frame_times,
            draw_times,
        })
    }

    pub fn draw_timer(&self) -> &RefCell<DrawTimer> {
        &*self.draw_times
    }

    pub fn frame_guard(&mut self, dt_secs: f32) -> FrameGuard {
        let start_time = Instant::now();

        let is_outdated = |(time, _): &(Instant, Duration)| {
            start_time.duration_since(*time) > self.params.plot_duration
        };

        self.dts
            .push_back((start_time, Duration::from_secs_f32(dt_secs)));
        while self.dts.front().map_or(false, is_outdated) {
            self.dts.pop_front();
        }

        while self.frame_times.borrow().front().map_or(false, is_outdated) {
            self.frame_times.borrow_mut().pop_front();
        }

        self.draw_times.borrow_mut().start_draw();

        FrameGuard {
            start_time: Instant::now(),
            _profile_guard: coarse_prof::enter("frame"),
            frame_times: self.frame_times.clone(),
            draw_times: self.draw_times.clone(),
        }
    }

    pub fn draw(&mut self, screen: Screen) -> Result<(), FrameError> {
        coarse_prof::profile!("Profile::draw");

        self.render(screen)?;
        self.pass
            .draw(&self.screen_matrices, &mut self.font, &mut self.batch);

        Ok(())
    }

    fn render(&mut self, screen: Screen) -> Result<(), FrameError> {
        coarse_prof::profile!("Profile::render");

        self.screen_matrices.set(MatricesBlock {
            view: Matrix3::identity(),
            projection: screen.orthographic_projection(),
        });

        self.batch.clear();

        let prof_string = coarse_prof::to_string();
        let prof_size =
            self.font.text_size(self.params.text_size, &prof_string) + 2.0 * self.params.padding;
        let prof_pos = Point2::from(screen.logical_size) - prof_size - self.params.margin;

        self.font.write(
            Text {
                pos: prof_pos + self.params.padding,
                size: self.params.text_size,
                z: 0.0,
                color: Color4::new(0.0, 0.0, 0.0, 1.0),
                text: &prof_string,
            },
            &mut self.batch.text_batch,
        )?;

        self.batch.triangle_batch.push(ColorRect {
            rect: Rect::from_top_left(prof_pos, prof_size),
            z: 0.0,
            color: PlotStyle::default().background_color.unwrap(),
        });

        let plot = self.plot(Rect::from_bottom_left(
            screen.logical_rect().bottom_left()
                + Vector2::new(self.params.margin.x, -self.params.margin.y),
            self.params.plot_size,
        ));
        self.batch
            .push(&mut self.font, plot, PlotStyle::default())?;

        Ok(())
    }

    fn plot(&self, rect: Rect) -> Plot {
        let mut line_graphs = Vec::new();
        if let Some((last_time, _)) = self.dts.back() {
            let point_pos = |(time, dur): &(Instant, Duration)| {
                (
                    -last_time.duration_since(*time).as_secs_f32(),
                    dur.as_secs_f32() * 1000.0,
                )
            };

            line_graphs.push(LineGraph {
                caption: "dt[ms]".to_owned(),
                color: Color4::new(0.0, 1.0, 0.0, 1.0),
                points: self.dts.iter().map(point_pos).collect(),
            });
            line_graphs.push(LineGraph {
                caption: "frame[ms]".to_owned(),
                color: Color4::new(1.0, 0.0, 0.0, 1.0),
                points: self.frame_times.borrow().iter().map(point_pos).collect(),
            });
            line_graphs.push(LineGraph {
                caption: "draw[ms]".to_owned(),
                color: Color4::new(0.0, 0.0, 1.0, 1.0),
                points: self
                    .draw_times
                    .borrow()
                    .samples()
                    .iter()
                    .map(point_pos)
                    .collect(),
            });
        }

        Plot {
            rect,
            x_axis: Axis {
                label: "dt[s]".to_owned(),
                range: Some((-self.params.plot_duration.as_secs_f32(), 0.0)),
                tics: 1.0,
            },
            y_axis: Axis {
                label: "dur[ms]".to_owned(),
                //range: Some((0.0, 30.0)),
                range: None,
                tics: 15.0,
            },
            line_graphs: line_graphs,
        }
    }
}

impl Drop for FrameGuard {
    fn drop(&mut self) {
        coarse_prof::profile!("FrameGuard::drop");

        let mut frame_times = self.frame_times.borrow_mut();
        frame_times.push_back((
            self.start_time,
            Instant::now().duration_since(self.start_time),
        ));

        let mut draw_times = self.draw_times.borrow_mut();
        draw_times.end_draw();
    }
}
