//! IPC(interprocess communication) related components.
use std::fmt;

use {Message, System};

/// Messages used for interprocess communications.
#[allow(missing_docs)]
pub enum IpcMessage<T: System> {
    Gossip(GossipMessage<T>),
    Ihave(IhaveMessage<T>),
    Graft(GraftMessage<T>),
    Prune(PruneMessage<T>),
}
impl<T: System> IpcMessage<T> {
    /// Returns the sender of the message.
    pub fn sender(&self) -> &T::NodeId {
        match self {
            IpcMessage::Gossip(m) => &m.sender,
            IpcMessage::Ihave(m) => &m.sender,
            IpcMessage::Graft(m) => &m.sender,
            IpcMessage::Prune(m) => &m.sender,
        }
    }
}
impl<T: System> Clone for IpcMessage<T> {
    fn clone(&self) -> Self {
        match self {
            IpcMessage::Gossip(m) => m.clone().into(),
            IpcMessage::Ihave(m) => m.clone().into(),
            IpcMessage::Graft(m) => m.clone().into(),
            IpcMessage::Prune(m) => m.clone().into(),
        }
    }
}
impl<T: System> fmt::Debug for IpcMessage<T>
where
    T::NodeId: fmt::Debug,
    T::MessageId: fmt::Debug,
    T::MessagePayload: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IpcMessage::Gossip(m) => write!(f, "Gossip({:?})", m),
            IpcMessage::Ihave(m) => write!(f, "Ihave({:?})", m),
            IpcMessage::Graft(m) => write!(f, "Graft({:?})", m),
            IpcMessage::Prune(m) => write!(f, "Prune({:?})", m),
        }
    }
}
impl<T: System> From<GossipMessage<T>> for IpcMessage<T> {
    fn from(f: GossipMessage<T>) -> Self {
        IpcMessage::Gossip(f)
    }
}
impl<T: System> From<IhaveMessage<T>> for IpcMessage<T> {
    fn from(f: IhaveMessage<T>) -> Self {
        IpcMessage::Ihave(f)
    }
}
impl<T: System> From<GraftMessage<T>> for IpcMessage<T> {
    fn from(f: GraftMessage<T>) -> Self {
        IpcMessage::Graft(f)
    }
}
impl<T: System> From<PruneMessage<T>> for IpcMessage<T> {
    fn from(f: PruneMessage<T>) -> Self {
        IpcMessage::Prune(f)
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
