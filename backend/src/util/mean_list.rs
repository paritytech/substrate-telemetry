pub struct MeanList {
    period_sum: f32,
    period_count: u8,
    mean_index: u8,
    means: [f32; 20],
    ticks_per_mean: u8,
}

impl MeanList {
    pub fn new() -> MeanList {
        MeanList {
            period_sum: 0.0,
            period_count: 0,
            mean_index: 0,
            means: [0.0; 20],
            ticks_per_mean: 1,
        }
    }

    pub fn slice(&self) -> &[f32] {
        &self.means[..usize::from(self.mean_index)]
    }

    pub fn push(&mut self, val: f32) -> bool {
        if self.mean_index == 20 && self.ticks_per_mean < 32 {
            self.squash_means();
        }

        self.period_sum += val;
        self.period_count += 1;

        if self.period_count == self.ticks_per_mean {
            self.push_mean();
            true
        } else {
            false
        }
    }

    fn push_mean(&mut self) {
        let mean = self.period_sum / self.period_count as f32;

        if self.mean_index == 20 && self.ticks_per_mean == 32 {
            self.means.rotate_left(1);
            self.means[19] = mean;
        } else {
            self.means[usize::from(self.mean_index)] = mean;
            self.mean_index += 1;
        }

        self.period_sum = 0.0;
        self.period_count = 0;
    }

    fn squash_means(&mut self) {
        self.ticks_per_mean *= 2;
        self.mean_index = 10;

        for i in 0..10 {
            let i2 = i * 2;

            self.means[i] = (self.means[i2] + self.means[i2 + 1]) / 2.0
        }
    }
}
