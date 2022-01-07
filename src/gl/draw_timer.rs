use std::{collections::VecDeque, rc::Rc, time::Duration};

use glow::HasContext;
use instant::Instant;

use super::Context;

const MAX_POLL_QUERIES: usize = 100;

pub struct DrawTimer {
    gl: Rc<Context>,
    max_age: Duration,
    is_supported: bool,

    last_query: Option<(Instant, glow::Query)>,
    poll_queries: VecDeque<(Instant, glow::Query)>,

    samples: VecDeque<(Instant, Duration)>,
}

#[derive(Debug, Clone)]
pub struct DrawTimingInfo {
    pub num_samples: usize,
    pub average: Duration,
    pub min: Duration,
    pub max: Duration,
}

// https://www.khronos.org/registry/webgl/extensions/EXT_disjoint_timer_query_webgl2/

impl DrawTimer {
    pub fn new(gl: Rc<Context>, max_age: Duration) -> Self {
        let is_supported = gl
            .supported_extensions()
            .contains("EXT_disjoint_timer_query_webgl2");

        log::info!("{:?}", gl.supported_extensions());

        Self {
            gl,
            max_age,
            is_supported,
            last_query: None,
            poll_queries: VecDeque::new(),
            samples: VecDeque::new(),
        }
    }

    pub fn start_draw(&mut self) {
        if !self.is_supported {
            return;
        }

        assert!(
            !self.last_query.is_some(),
            "end_draw must be called after start_draw"
        );

        if !self.poll_queries.is_empty() {
            return;
        }

        if let Ok(query) = unsafe { self.gl.create_query() } {
            unsafe {
                self.gl.begin_query(glow::TIME_ELAPSED, query);
            }

            let current_time = Instant::now();
            self.last_query = Some((current_time, query));

            while self.samples.front().map_or(false, |(time, _)| {
                current_time.duration_since(*time) > self.max_age
            }) {
                self.samples.pop_front();
            }
        }
    }

    pub fn end_draw(&mut self) {
        if !self.is_supported {
            return;
        }

        if let Some(last_query) = self.last_query {
            unsafe {
                self.gl.end_query(glow::TIME_ELAPSED);
            }

            self.poll_queries.push_back(last_query);

            if self.poll_queries.len() > MAX_POLL_QUERIES {
                self.poll_queries.pop_front();
            }

            self.last_query = None;
        }

        // FIXME: Somehow QUERY_RESULT_AVAILABLE always returns zero for me!

        while let Some(&(time, available_query)) = self.poll_queries.front().filter(
            |&&(_, poll_query)|
                unsafe { self.gl.get_query_parameter_u32(poll_query, glow::QUERY_RESULT) }
                != 0
        ) {
            let nanos = unsafe {
                self.gl.get_query_parameter_u32(available_query, glow::QUERY_RESULT)
            };
            self.samples.push_back((time, Duration::from_nanos(nanos as u64)));

            unsafe { self.gl.delete_query(available_query); }
            self.poll_queries.pop_front();
        }
    }

    pub fn samples(&self) -> &VecDeque<(Instant, Duration)> {
        &self.samples
    }

    pub fn timing_info(&self) -> Option<DrawTimingInfo> {
        if self.samples.is_empty() {
            return None;
        }

        Some(DrawTimingInfo {
            num_samples: self.samples.len(),
            average: Duration::from_nanos(
                (self
                    .samples
                    .iter()
                    .map(|(_, dur)| dur.as_nanos() as f64)
                    .sum::<f64>()
                    / self.samples.len() as f64) as u64,
            ),
            min: self.samples.iter().map(|(_, dur)| *dur).min().unwrap(),
            max: self.samples.iter().map(|(_, dur)| *dur).max().unwrap(),
        })
    }
}

impl Drop for DrawTimer {
    fn drop(&mut self) {
        for (_, query) in &self.poll_queries {
            unsafe {
                self.gl.delete_query(*query);
            }
        }
    }
}
