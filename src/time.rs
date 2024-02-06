use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TickTimer {
    first: Instant,
    last_elapsed: Duration,
}

impl Default for TickTimer {
    fn default() -> Self {
        Self {
            first: Instant::now(),
            last_elapsed: Duration::ZERO,
        }
    }
}

impl TickTimer {
    pub fn total_elapsed(&self) -> Duration {
        self.first.elapsed()
    }

    pub fn tick(&mut self) -> Duration {
        let elapsed = self.first.elapsed();
        let tick_duration = elapsed - self.last_elapsed;
        self.last_elapsed = elapsed;
        tick_duration
    }
}

#[derive(Default, Debug)]
pub struct FrameTiming {
    tick_timer: TickTimer,
    delta: Duration,
    frame_counter: usize,
    frame_timer: Duration,
    sampled_fps: f32,
}

impl FrameTiming {
    pub fn update(&mut self) {
        self.frame_counter += 1;
        self.delta = self.tick_timer.tick();
        self.frame_timer += self.delta;
        if self.frame_counter >= 50 {
            self.sampled_fps = self.frame_counter as f32 / self.frame_timer.as_secs_f32();
            self.frame_counter = 0;
            self.frame_timer = Duration::ZERO;
        }
    }

    pub fn fps(&self) -> f32 {
        self.sampled_fps
    }

    pub fn time(&self) -> f32 {
        self.tick_timer.total_elapsed().as_secs_f32()
    }
}
