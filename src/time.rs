//! Node local clock and time.
use std;
use std::ops::{Add, AddAssign};
use std::time::Duration;

/// Node local clock.
///
/// Each [`Node`] has a clock instance.
/// When a node is created, the time of its clock is initialized to zero.
/// Then each time [`Clock::tick`] method is called, the time of the clock proceeds by the specified duration.
///
/// [`Node`]: ../struct.Node.html
/// [`Clock::tick`]: ./struct.Clock.html#method.tick
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Clock(Duration);
impl Clock {
    /// Makes a new `Clock` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use plumtree::time::Clock;
    /// use std::time::Duration;
    ///
    /// assert_eq!(Clock::new().now().as_duration(), Duration::from_secs(0));
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current time of the clock.
    ///
    /// # Examples
    ///
    /// ```
    /// use plumtree::time::Clock;
    /// use std::time::Duration;
    ///
    /// let mut clock = Clock::new();
    /// assert_eq!(clock.now().as_duration(), Duration::from_secs(0));
    ///
    /// clock.tick(Duration::from_secs(100));
    /// assert_eq!(clock.now().as_duration(), Duration::from_secs(100));
    /// ```
    pub fn now(&self) -> NodeTime {
        NodeTime(self.0)
    }

    /// Proceeds the time of the clock by the given duration.
    pub fn tick(&mut self, duration: Duration) {
        self.0 += duration;
    }

    pub(crate) fn max() -> Self {
        let max = Duration::MAX;
        Clock(max)
    }
}

/// Node local time.
///
/// This represents the elapsed logical time since a clock was created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeTime(Duration);
impl NodeTime {
    /// Converts `NodeTime` to `Duration`.
    pub fn as_duration(&self) -> Duration {
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
