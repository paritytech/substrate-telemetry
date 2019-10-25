use num_traits::{Zero, NumOps, Bounded};
use std::iter::Sum;
use std::convert::TryFrom;

/// Keep track of last N numbers pushed onto internal stack.
/// Provides means to get an average of said numbers.
pub struct NumStats<T> {
    stack: Box<[T]>,
    index: usize,
    sum: T,
}

impl<T: NumOps + Zero + Bounded + Copy + Sum + TryFrom<usize>> NumStats<T> {
    pub fn new(size: usize) -> Self {
        NumStats {
            stack: std::iter::repeat(T::zero()).take(size).collect(),
            index: 0,
            sum: T::zero(),
        }
    }

    pub fn push(&mut self, val: T) {
        let slot = &mut self.stack[self.index % self.stack.len()];

        self.sum = self.sum - *slot + val;

        *slot = val;

        self.index += 1;
    }

    pub fn average(&self) -> T {
        let cap = std::cmp::min(self.index, self.stack.len());
        let cap = T::try_from(cap).unwrap_or_else(|_| T::max_value());

        self.sum / cap
    }
}
