use System;

#[derive(Clone)]
pub enum Message<T: System> {
    Gossip(GossipMessage<T>),
    Ihave(IhaveMessage<T>),
    Graft(GraftMessage<T>),
    Prune(PruneMessage<T>),
}
impl<T: System> Message<T> {
    pub fn sender(&self) -> &T::NodeId {
        match self {
            Message::Gossip(m) => &m.sender,
            Message::Ihave(m) => &m.sender,
            Message::Graft(m) => &m.sender,
            Message::Prune(m) => &m.sender,
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

pub struct GossipMessage<T: System> {
    pub sender: T::NodeId,
    pub message_id: T::MessageId,
    pub round: u16,
    // TODO: payload
}
impl<T: System> Clone for GossipMessage<T> {
    fn clone(&self) -> Self {
        GossipMessage {
            sender: self.sender.clone(),
            message_id: self.message_id.clone(),
            round: self.round,
        }
    }
}

// It is allowed to delay sending this message arbitrary time within ...
pub struct IhaveMessage<T: System> {
    pub sender: T::NodeId,
    pub message_id: T::MessageId,
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

#[derive(Clone)]
pub struct GraftMessage<T: System> {
    pub sender: T::NodeId,
    pub message_id: Option<T::MessageId>,
    pub round: u16,
}

#[derive(Clone)]
pub struct PruneMessage<T: System> {
    pub sender: T::NodeId,
}
impl<T: System> PruneMessage<T> {
    pub(crate) fn new(sender: &T::NodeId) -> Self {
        PruneMessage {
            sender: sender.clone(),
        }
    }
}
