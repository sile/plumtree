#[derive(Debug, Clone)]
pub enum Message<N, M> {
    Gossip(GossipMessage<N, M>),
    Ihave(IhaveMessage<N, M>),
    Graft(GraftMessage<N, M>),
    Prune(PruneMessage<N>),
}
impl<N, M> Message<N, M> {
    pub fn sender(&self) -> &N {
        match self {
            Message::Gossip(m) => &m.sender,
            Message::Ihave(m) => &m.sender,
            Message::Graft(m) => &m.sender,
            Message::Prune(m) => &m.sender,
        }
    }
}
impl<N, M> From<GossipMessage<N, M>> for Message<N, M> {
    fn from(f: GossipMessage<N, M>) -> Self {
        Message::Gossip(f)
    }
}
impl<N, M> From<IhaveMessage<N, M>> for Message<N, M> {
    fn from(f: IhaveMessage<N, M>) -> Self {
        Message::Ihave(f)
    }
}
impl<N, M> From<GraftMessage<N, M>> for Message<N, M> {
    fn from(f: GraftMessage<N, M>) -> Self {
        Message::Graft(f)
    }
}
impl<N, M> From<PruneMessage<N>> for Message<N, M> {
    fn from(f: PruneMessage<N>) -> Self {
        Message::Prune(f)
    }
}

#[derive(Debug, Clone)]
pub struct GossipMessage<N, M> {
    pub sender: N,
    pub message_id: M,
    pub round: u16,
    // TODO: payload
}

// It is allowed to delay sending this message arbitrary time within ...
#[derive(Debug, Clone)]
pub struct IhaveMessage<N, M> {
    pub sender: N,
    pub message_id: M,
    pub round: u16,
}

#[derive(Debug, Clone)]
pub struct GraftMessage<N, M> {
    pub sender: N,
    pub message_id: Option<M>,
    pub round: u16,
}

#[derive(Debug, Clone)]
pub struct PruneMessage<N> {
    pub sender: N,
}
impl<N: Clone> PruneMessage<N> {
    pub(crate) fn new(sender: &N) -> Self {
        PruneMessage {
            sender: sender.clone(),
        }
    }
}
