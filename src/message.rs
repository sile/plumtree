use std::fmt;

use System;

/// Messages used for inter-node communication.
#[allow(missing_docs)]
pub enum Message<T: System> {
    Gossip(GossipMessage<T>),
    Ihave(IhaveMessage<T>),
    Graft(GraftMessage<T>),
    Prune(PruneMessage<T>),
}
impl<T: System> Message<T> {
    /// Returns the sender of the message.
    pub fn sender(&self) -> &T::NodeId {
        match self {
            Message::Gossip(m) => &m.sender,
            Message::Ihave(m) => &m.sender,
            Message::Graft(m) => &m.sender,
            Message::Prune(m) => &m.sender,
        }
    }
}
impl<T: System> Clone for Message<T> {
    fn clone(&self) -> Self {
        match self {
            Message::Gossip(m) => m.clone().into(),
            Message::Ihave(m) => m.clone().into(),
            Message::Graft(m) => m.clone().into(),
            Message::Prune(m) => m.clone().into(),
        }
    }
}
impl<T: System> fmt::Debug for Message<T>
where
    T::NodeId: fmt::Debug,
    T::MessageId: fmt::Debug,
    T::MessagePayload: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Message::Gossip(m) => write!(f, "Gossip({:?})", m),
            Message::Ihave(m) => write!(f, "Ihave({:?})", m),
            Message::Graft(m) => write!(f, "Graft({:?})", m),
            Message::Prune(m) => write!(f, "Prune({:?})", m),
        }
    }
}
impl<T: System> From<GossipMessage<T>> for Message<T> {
    fn from(f: GossipMessage<T>) -> Self {
        Message::Gossip(f)
    }
}
impl<T: System> From<IhaveMessage<T>> for Message<T> {
    fn from(f: IhaveMessage<T>) -> Self {
        Message::Ihave(f)
    }
}
impl<T: System> From<GraftMessage<T>> for Message<T> {
    fn from(f: GraftMessage<T>) -> Self {
        Message::Graft(f)
    }
}
impl<T: System> From<PruneMessage<T>> for Message<T> {
    fn from(f: PruneMessage<T>) -> Self {
        Message::Prune(f)
    }
}

/// `GOSSIP` message.
pub struct GossipMessage<T: System> {
    /// The sender of the message.
    pub sender: T::NodeId,

    /// The identifier of the message.
    pub message_id: T::MessageId,

    /// The payload of the message.
    pub message_payload: T::MessagePayload,

    /// The hop count of the message.
    pub round: u16,
}
impl<T: System> Clone for GossipMessage<T> {
    fn clone(&self) -> Self {
        GossipMessage {
            sender: self.sender.clone(),
            message_id: self.message_id.clone(),
            message_payload: self.message_payload.clone(),
            round: self.round,
        }
    }
}
impl<T: System> fmt::Debug for GossipMessage<T>
where
    T::NodeId: fmt::Debug,
    T::MessageId: fmt::Debug,
    T::MessagePayload: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "GossipMessage {{ sender: {:?}, message_id: {:?}, message_payload: {:?}, round: {:?} }}",
             self.sender, self.message_id, self.message_payload, self.round
        )
    }
}

/// `IHAVE` message.
pub struct IhaveMessage<T: System> {
    /// The sender of the message.
    pub sender: T::NodeId,

    /// The identifier of the message that the sender has keeping.
    pub message_id: T::MessageId,

    /// The hop count of the message.
    pub round: u16,
}
impl<T: System> Clone for IhaveMessage<T> {
    fn clone(&self) -> Self {
        IhaveMessage {
            sender: self.sender.clone(),
            message_id: self.message_id.clone(),
            round: self.round,
        }
    }
}
impl<T: System> fmt::Debug for IhaveMessage<T>
where
    T::NodeId: fmt::Debug,
    T::MessageId: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "IhaveMessage {{ sender: {:?}, message_id: {:?}, round: {:?} }}",
            self.sender, self.message_id, self.round
        )
    }
}

/// `GRAFT` message.
pub struct GraftMessage<T: System> {
    /// The sender of the message.
    pub sender: T::NodeId,

    /// The identifier of the message requested by the sender.
    pub message_id: Option<T::MessageId>,

    /// The hop count of the message.
    pub round: u16,
}
impl<T: System> Clone for GraftMessage<T> {
    fn clone(&self) -> Self {
        GraftMessage {
            sender: self.sender.clone(),
            message_id: self.message_id.clone(),
            round: self.round,
        }
    }
}
impl<T: System> fmt::Debug for GraftMessage<T>
where
    T::NodeId: fmt::Debug,
    T::MessageId: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "GraftMessage {{ sender: {:?}, message_id: {:?}, round: {:?} }}",
            self.sender, self.message_id, self.round
        )
    }
}

/// `PRUNE` message.
pub struct PruneMessage<T: System> {
    /// The sender of the message.
    pub sender: T::NodeId,
}
impl<T: System> PruneMessage<T> {
    pub(crate) fn new(sender: &T::NodeId) -> Self {
        PruneMessage {
            sender: sender.clone(),
        }
    }
}
impl<T: System> Clone for PruneMessage<T> {
    fn clone(&self) -> Self {
        PruneMessage {
            sender: self.sender.clone(),
        }
    }
}
impl<T: System> fmt::Debug for PruneMessage<T>
where
    T::NodeId: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PruneMessage {{ sender: {:?} }}", self.sender)
    }
}
