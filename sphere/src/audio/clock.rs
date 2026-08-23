use std::sync::atomic::{AtomicI64, Ordering};

use common::time::{TimeBase, Timestamp};

pub struct AudioClock {
    samples_played: AtomicI64,
    sample_rate: u32,
}

impl AudioClock {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            samples_played: AtomicI64::new(0),
            sample_rate,
        }
    }

    pub fn current_time(&self) -> Timestamp {
        let samples = self.samples_played.load(Ordering::Relaxed);
        Timestamp::new(samples, TimeBase::new(1, self.sample_rate))
    }

    pub fn advance_by_samples(&self, sample_count: usize) {
        self.samples_played
            .fetch_add(sample_count as i64, Ordering::Relaxed);
    }
}
