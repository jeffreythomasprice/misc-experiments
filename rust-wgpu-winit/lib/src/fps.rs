use std::{collections::VecDeque, time::Duration};

const MAX_BUCKET_DURATION: Duration = Duration::from_millis(500);
const MAX_BUCKET_TICKS: u64 = 100;
const MAX_BUCKETS: usize = 3;

struct Bucket {
    duration: Duration,
    ticks: u64,
}

pub struct FPSCounter {
    buckets: VecDeque<Bucket>,
    total_duration: Duration,
    total_ticks: u64,
}

impl FPSCounter {
    pub fn new() -> Self {
        Self {
            buckets: VecDeque::with_capacity(MAX_BUCKETS),
            total_duration: Duration::ZERO,
            total_ticks: 0,
        }
    }

    pub fn tick(&mut self, duration: Duration) {
        if let Some(current_bucket) = self.buckets.back_mut() {
            // we have something already, see if we can add to the current bucket or need to make a new one
            let next_duration = current_bucket.duration + duration;
            let next_ticks = current_bucket.ticks + 1;
            if next_duration > MAX_BUCKET_DURATION || next_ticks > MAX_BUCKET_TICKS {
                // bucket is full, we need to move to a new bucket
                // first check if we need to remove an existing bucket
                if self.buckets.len() >= MAX_BUCKETS {
                    if let Some(removed_bucket) = self.buckets.pop_front() {
                        self.total_duration -= removed_bucket.duration;
                        self.total_ticks -= removed_bucket.ticks;
                    }
                }
                // now add the new bucket
                self.buckets.push_back(Bucket { duration, ticks: 1 });
                self.total_duration += duration;
                self.total_ticks += 1;
            } else {
                // bucket is not full, so save that
                current_bucket.duration = next_duration;
                current_bucket.ticks = next_ticks;
                self.total_duration += duration;
                self.total_ticks += 1;
            }
        } else {
            // empty, this is the first tick
            self.buckets.push_back(Bucket { duration, ticks: 1 });
            self.total_duration += duration;
            self.total_ticks += 1;
        }
    }

    pub fn total_duration(&self) -> &Duration {
        &self.total_duration
    }

    pub fn total_ticks(&self) -> u64 {
        self.total_ticks
    }

    pub fn fps(&self) -> f64 {
        (self.total_ticks as f64) / self.total_duration.as_secs_f64()
    }

    pub fn fps_pretty(&self) -> String {
        format!("{:.1}", (self.fps() * 10.0).round() / 10.0)
    }
}
