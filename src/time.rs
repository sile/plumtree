use std::ops::{Add, AddAssign};
use std::time::Duration;

// TODO: /// Node local logical time.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Clock(Duration);
impl Clock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self, duration: Duration) {
        self.0 += duration;
    }

    pub fn now(&self) -> NodeTime {
        NodeTime(self.0)
    }

    pub fn elapsed(&self) -> Duration {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeTime(Duration);
impl NodeTime {
    pub fn elapsed(&self) -> Duration {
        self.0
    }
}
impl Add<Duration> for NodeTime {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        NodeTime(self.0 + rhs)
    }
}
impl AddAssign<Duration> for NodeTime {
    fn add_assign(&mut self, rhs: Duration) {
        self.0 += rhs;
    }
}
