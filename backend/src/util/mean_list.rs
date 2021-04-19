use num_traits::{Float, Zero};
use std::ops::AddAssign;

pub struct MeanList<T>
where
    T: Float + AddAssign + Zero + From<u8>,
{
    period_sum: T,
    period_count: u8,
    mean_index: u8,
    means: [T; 20],
    ticks_per_mean: u8,
}

impl<T> Default for MeanList<T>
where
    T: Float + AddAssign + Zero + From<u8>,
{
    fn default() -> MeanList<T> {
        MeanList {
            period_sum: T::zero(),
            period_count: 0,
            mean_index: 0,
            means: [T::zero(); 20],
            ticks_per_mean: 1,
        }
    }
}

impl<T> MeanList<T>
where
    T: Float + AddAssign + Zero + From<u8>,
{
    pub fn slice(&self) -> &[T] {
        &self.means[..usize::from(self.mean_index)]
    }

    pub fn push(&mut self, val: T) -> bool {
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
        let mean = self.period_sum / std::convert::From::from(self.period_count);

        if self.mean_index == 20 && self.ticks_per_mean == 32 {
            self.means.rotate_left(1);
            self.means[19] = mean;
        } else {
            self.means[usize::from(self.mean_index)] = mean;
            self.mean_index += 1;
        }

        self.period_sum = T::zero();
        self.period_count = 0;
    }

    fn squash_means(&mut self) {
        self.ticks_per_mean *= 2;
        self.mean_index = 10;

        for i in 0..10 {
            let i2 = i * 2;

            self.means[i] = (self.means[i2] + self.means[i2 + 1]) / std::convert::From::from(2)
        }
    }
}
