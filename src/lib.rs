//! A Rust implementation of [Plumtree] algorithm.
//!
//! # References
//!
//! - [Plumtree: Epidemic Broadcast Trees][Plumtree]
//!
//! [Plumtree]: http://www.gsd.inesc-id.pt/~ler/reports/srds07.pdf
// TODO #![warn(missing_docs)]
pub use action::Action;
pub use node::{Node, NodeOptions};
pub use system::System;

mod action;
mod missing;
mod node;
mod system;

pub mod message;
pub mod time;
