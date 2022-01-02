use std::{collections::VecDeque, rc::Rc, time::Duration};

use glow::HasContext;

use super::Context;

pub struct FrameTimer {
    gl: Rc<Context>,
    max_samples: usize,
    is_supported: bool,

    last_query: Option<<glow::Context as HasContext>::Query>,
    poll_queries: VecDeque<<glow::Context as HasContext>::Query>,

    sample_nanos: VecDeque<u64>,
}

#[derive(Debug, Clone)]
pub struct TimingInfo {
    pub num_samples: usize,
    pub average: Duration,
    pub min: Duration,
    pub max: Duration,
}

// https://www.khronos.org/registry/webgl/extensions/EXT_disjoint_timer_query_webgl2/

impl FrameTimer {
    pub fn new(gl: Rc<Context>, max_samples: usize) -> Self {
        let is_supported = gl
            .supported_extensions()
            .contains("EXT_disjoint_timer_query_webgl2");

        log::info!("{:?}", gl.supported_extensions());

        Self {
            gl,
            max_samples,
            is_supported,
            last_query: None,
            poll_queries: VecDeque::new(),
            sample_nanos: VecDeque::new(),
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

            self.last_query = Some(query);
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
            if self.poll_queries.len() > self.max_samples {
                self.poll_queries.pop_front();
            }

            self.last_query = None;
        }

        // FIXME: Somehow QUERY_RESULT_AVAILABLE always returns zero for me!

        while let Some(&available_query) = self.poll_queries.front().filter(
            |&&poll_query|
                unsafe { self.gl.get_query_parameter_u32(poll_query, glow::QUERY_RESULT) }
                != 0
        ) {
            let nanos = unsafe {
                self.gl.get_query_parameter_u32(available_query, glow::QUERY_RESULT)
            };
            self.sample_nanos.push_back(nanos as u64);
            if self.sample_nanos.len() > self.max_samples {
                self.sample_nanos.pop_front();
            }

            unsafe { self.gl.delete_query(available_query); }
            self.poll_queries.pop_front();
        }
    }

    pub fn timing_info(&self) -> Option<TimingInfo> {
        if self.sample_nanos.is_empty() {
            return None;
        }

        Some(TimingInfo {
            num_samples: self.sample_nanos.len(),
            average: Duration::from_nanos(
                (self
                    .sample_nanos
                    .iter()
                    .map(|&nanos| nanos as f64)
                    .sum::<f64>()
                    / self.sample_nanos.len() as f64) as u64,
            ),
            min: self
                .sample_nanos
                .iter()
                .copied()
                .map(Duration::from_nanos)
                .min()
                .unwrap(),
            max: self
                .sample_nanos
                .iter()
                .copied()
                .map(Duration::from_nanos)
                .max()
                .unwrap(),
        })
    }
}

impl Drop for FrameTimer {
    fn drop(&mut self) {
        for query in &self.poll_queries {
            unsafe {
                self.gl.delete_query(*query);
            }
        }
    }
}
