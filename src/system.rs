use std::hash::Hash;

/// This trait allows for defining a system to which Plumtree nodes belong.
pub trait System {
    /// Node identifier.
    type NodeId: Clone + Hash + Eq;

    /// Message identifier.
    type MessageId: Clone + Hash + Eq;

    /// Message payload.
    type MessagePayload: Clone;
}
