//! Application and protocol messages.
use crate::System;
use std::fmt;

/// Application message.
pub struct Message<T: System> {
    /// The identifier of the message.
    pub id: T::MessageId,

    /// The payload of the message
    pub payload: T::MessagePayload,
}
impl<T: System> Message<T> {
    /// Makes a new `Message` instance.
    ///
    /// This is equivalent to `Message { id, payload }`.
    pub fn new(id: T::MessageId, payload: T::MessagePayload) -> Self {
        Message { id, payload }
    }
}
impl<T: System> Clone for Message<T> {
    fn clone(&self) -> Self {
        Message {
            id: self.id.clone(),
            payload: self.payload.clone(),
        }
    }
}
impl<T: System> fmt::Debug for Message<T>
where
    T::MessageId: fmt::Debug,
    T::MessagePayload: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Message {{ id: {:?}, payload: {:?} }}",
            self.id, self.payload
        )
    }
}
impl<T: System> PartialEq for Message<T>
where
    T::MessageId: PartialEq,
    T::MessagePayload: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id) && self.payload.eq(&other.payload)
    }
}
impl<T: System> Eq for Message<T>
where
    T::MessageId: Eq,
    T::MessagePayload: Eq,
{
}

/// Messages defined by the Plumtree algorithm.
///
/// Those are used for inter-node communications.
#[allow(missing_docs)]
pub enum ProtocolMessage<T: System> {
    Gossip(GossipMessage<T>),
    Ihave(IhaveMessage<T>),
    Graft(GraftMessage<T>),
    Prune(PruneMessage<T>),
}
impl<T: System> ProtocolMessage<T> {
    /// Returns the sender of the message.
    pub fn sender(&self) -> &T::NodeId {
        match self {
            ProtocolMessage::Gossip(m) => &m.sender,
            ProtocolMessage::Ihave(m) => &m.sender,
            ProtocolMessage::Graft(m) => &m.sender,
            ProtocolMessage::Prune(m) => &m.sender,
        }
    }
}
impl<T: System> Clone for ProtocolMessage<T> {
    fn clone(&self) -> Self {
        match self {
            ProtocolMessage::Gossip(m) => m.clone().into(),
            ProtocolMessage::Ihave(m) => m.clone().into(),
            ProtocolMessage::Graft(m) => m.clone().into(),
            ProtocolMessage::Prune(m) => m.clone().into(),
        }
    }
}
impl<T: System> fmt::Debug for ProtocolMessage<T>
where
    T::NodeId: fmt::Debug,
    T::MessageId: fmt::Debug,
    T::MessagePayload: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProtocolMessage::Gossip(m) => write!(f, "Gossip({:?})", m),
            ProtocolMessage::Ihave(m) => write!(f, "Ihave({:?})", m),
            ProtocolMessage::Graft(m) => write!(f, "Graft({:?})", m),
            ProtocolMessage::Prune(m) => write!(f, "Prune({:?})", m),
        }
    }
}
impl<T: System> From<GossipMessage<T>> for ProtocolMessage<T> {
    fn from(f: GossipMessage<T>) -> Self {
        ProtocolMessage::Gossip(f)
    }
}
impl<T: System> From<IhaveMessage<T>> for ProtocolMessage<T> {
    fn from(f: IhaveMessage<T>) -> Self {
        ProtocolMessage::Ihave(f)
    }
}
impl<T: System> From<GraftMessage<T>> for ProtocolMessage<T> {
    fn from(f: GraftMessage<T>) -> Self {
        ProtocolMessage::Graft(f)
    }
}
impl<T: System> From<PruneMessage<T>> for ProtocolMessage<T> {
    fn from(f: PruneMessage<T>) -> Self {
        ProtocolMessage::Prune(f)
    }
}

/// `GOSSIP` message.
pub struct GossipMessage<T: System> {
    /// The sender of the message.
    pub sender: T::NodeId,

    /// The message to be diffused.
    pub message: Message<T>,

    /// The hop count of the message.
    pub round: u16,
}
impl<T: System> GossipMessage<T> {
    pub(crate) fn new(sender: &T::NodeId, message: Message<T>, round: u16) -> Self {
        GossipMessage {
            sender: sender.clone(),
            message,
            round,
        }
    }
}
impl<T: System> Clone for GossipMessage<T> {
    fn clone(&self) -> Self {
        GossipMessage {
            sender: self.sender.clone(),
            message: self.message.clone(),
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
            "GossipMessage {{ sender: {:?}, message: {:?}, round: {:?} }}",
            self.sender, self.message, self.round
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

    /// Indicates whether this is a real-time message or a buffered message.
    ///
    /// The latter is used for synchronizing messages when new neighbors are joined.
    pub realtime: bool,
}
impl<T: System> IhaveMessage<T> {
    pub(crate) fn new(
        sender: &T::NodeId,
        message_id: T::MessageId,
        round: u16,
        realtime: bool,
    ) -> Self {
        IhaveMessage {
            sender: sender.clone(),
            message_id,
            round,
            realtime,
        }
    }
}
impl<T: System> Clone for IhaveMessage<T> {
    fn clone(&self) -> Self {
        IhaveMessage {
            sender: self.sender.clone(),
            message_id: self.message_id.clone(),
            round: self.round,
            realtime: self.realtime,
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
            "IhaveMessage {{ sender: {:?}, message_id: {:?}, round: {:?}, realtime: {:?} }}",
            self.sender, self.message_id, self.round, self.realtime
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
impl<T: System> GraftMessage<T> {
    pub(crate) fn new(sender: &T::NodeId, message_id: Option<T::MessageId>, round: u16) -> Self {
        GraftMessage {
            sender: sender.clone(),
            message_id,
            round,
        }
    }
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
