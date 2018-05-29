#[derive(Debug, Clone)]
pub enum Message<N, M> {
    Gossip(GossipMessage<N, M>),
    Ihave(IhaveMessage<N, M>),
    Graft(GraftMessage<N, M>),
    Prune(PruneMessage<N>),
}

#[derive(Debug, Clone)]
pub struct GossipMessage<N, M> {
    pub sender: N,
    pub message_id: M,
    pub round: u16,
}

#[derive(Debug, Clone)]
pub struct IhaveMessage<N, M> {
    pub sender: N,
    pub message_id: M,
    pub round: u16,
}

#[derive(Debug, Clone)]
pub struct GraftMessage<N, M> {
    pub sender: N,
    pub message_id: M,
    pub round: u16,
}

#[derive(Debug, Clone)]
pub struct PruneMessage<N> {
    pub sender: N,
}
