//! A Rust implementation of [Plumtree] algorithm.
//!
//! # References
//!
//! - [Plumtree: Epidemic Broadcast Trees][Plumtree]
//!
//! [Plumtree]: http://www.gsd.inesc-id.pt/~ler/reports/srds07.pdf
#![warn(missing_docs)]
pub use action::Action;
pub use node::{Node, NodeOptions};
pub use system::System;

mod action;
mod missing;
mod node;
mod system;
mod time;

pub mod message;
