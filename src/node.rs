use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::hash::Hash;

use action::{Action, ActionQueue};
use message::{GossipMessage, GraftMessage, IhaveMessage, Message, PruneMessage};

#[derive(Debug)]
pub struct Node<N, M>
where
    N: Hash + Eq,
    M: Hash + Eq,
{
    node_id: N,
    eager_push_peers: HashSet<N>, // TODO: Vec?
    lazy_push_peers: HashSet<N>,
    missing: MissingMessages<N, M>,
    received_msgs: HashSet<M>,
    action_queue: ActionQueue<N, M>,
    clock: u64,
}
impl<N, M> Node<N, M>
where
    N: Hash + Eq + Clone,
    M: Hash + Eq + Clone,
{
    pub fn new(node_id: N) -> Self {
        Node {
            node_id,
            eager_push_peers: HashSet::new(),
            lazy_push_peers: HashSet::new(),
            missing: MissingMessages::new(),
            received_msgs: HashSet::new(),
            action_queue: ActionQueue::new(),
            clock: 0,
        }
    }

    pub fn handle_message(&mut self, message: Message<N, M>) {
        if !self.is_known_node(message.sender()) {
            return;
        }
        match message {
            Message::Gossip(m) => self.handle_gossip(m),
            Message::Ihave(m) => self.handle_ihave(m),
            Message::Graft(m) => self.handle_graft(m),
            Message::Prune(m) => self.handle_prune(m),
        }
    }

    pub fn handle_neighbour_up(&mut self, neighbour_node_id: N) {
        if self.node_id == neighbour_node_id {
            // TODO: metrics
            return;
        }
        self.eager_push_peers.insert(neighbour_node_id);
    }

    pub fn handle_neighbour_down(&mut self, neighbour_node_id: N) {
        self.eager_push_peers.remove(&neighbour_node_id);
        self.lazy_push_peers.remove(&neighbour_node_id);
    }

    pub fn forget_message(&mut self, message_id: &M) {
        self.received_msgs.remove(message_id);
    }

    pub fn handle_tick(&mut self) {
        self.clock += 1;
        while let Some(ihave) = self.missing.pop_expired(self.clock) {
            if !self.is_known_node(&ihave.sender) {
                // The node has been removed from neighbours
                continue;
            }
            self.eager_push_peers.insert(ihave.sender.clone());
            self.lazy_push_peers.remove(&ihave.sender);
            self.action_queue.send(
                ihave.sender,
                GraftMessage {
                    sender: self.node_id.clone(),
                    message_id: Some(ihave.message_id),
                    round: ihave.round,
                },
            );
        }
    }

    pub fn poll_action(&mut self) -> Option<Action<N, M>> {
        self.action_queue.pop()
    }

    fn handle_gossip(&mut self, m: GossipMessage<N, M>) {
        if self.received_msgs.contains(&m.message_id) {
            self.eager_push_peers.remove(&m.sender);
            self.lazy_push_peers.insert(m.sender.clone());
            self.action_queue
                .send(m.sender, PruneMessage::new(&self.node_id));
        } else {
            self.action_queue.deliver(m.message_id.clone());
            self.received_msgs.insert(m.message_id.clone());
            self.missing.cancel_timer(&m.message_id);

            self.eager_push(m.clone());
            self.lazy_push(m.clone());
            self.eager_push_peers.insert(m.sender.clone());
            self.lazy_push_peers.remove(&m.sender);
            self.optimize(m);
        }
    }

    fn handle_ihave(&mut self, m: IhaveMessage<N, M>) {
        if self.received_msgs.contains(&m.message_id) {
            return;
        }
        let expiry_time = self.clock + 3; // TODO: parameter
        self.missing.push(m, expiry_time);
    }

    fn handle_graft(&mut self, mut m: GraftMessage<N, M>) {
        self.eager_push_peers.insert(m.sender.clone());
        self.lazy_push_peers.remove(&m.sender);
        if let Some(message_id) = m.message_id.take() {
            if self.received_msgs.contains(&message_id) {
                self.action_queue.send(
                    m.sender,
                    GossipMessage {
                        sender: self.node_id.clone(),
                        message_id,
                        round: m.round,
                    },
                );
            }
        }
    }

    fn handle_prune(&mut self, m: PruneMessage<N>) {
        self.eager_push_peers.remove(&m.sender);
        self.lazy_push_peers.insert(m.sender);
    }

    fn eager_push(&mut self, mut m: GossipMessage<N, M>) {
        let sender = m.sender;
        m.sender = self.node_id.clone();
        m.round = m.round.saturating_add(1);
        for p in self.eager_push_peers.iter().filter(|n| **n != sender) {
            self.action_queue.send(p.clone(), m.clone());
        }
    }

    fn lazy_push(&mut self, m: GossipMessage<N, M>) {
        let sender = m.sender;
        let m = IhaveMessage {
            sender: self.node_id.clone(),
            message_id: m.message_id,
            round: m.round.saturating_add(1),
        };
        for p in self.eager_push_peers.iter().filter(|n| **n != sender) {
            self.action_queue.send(p.clone(), m.clone());
        }
    }

    fn optimize(&mut self, m: GossipMessage<N, M>) {
        if let Some((round, node)) = self.missing.get_by_id(&m.message_id) {
            let threshold = 3; // TODO
            if round < m.round && (m.round - round) >= threshold {
                self.action_queue.send(
                    node.clone(),
                    GraftMessage {
                        sender: self.node_id.clone(),
                        message_id: None,
                        round,
                    },
                );
                self.action_queue
                    .send(node.clone(), PruneMessage::new(&self.node_id));
            }
        }
    }

    fn is_known_node(&self, node_id: &N) -> bool {
        self.eager_push_peers.contains(node_id) || self.lazy_push_peers.contains(node_id)
    }
}

#[derive(Debug)]
struct MissingMessages<N, M>
where
    M: Hash + Eq,
{
    ihaves: BinaryHeap<MissingMessage<N, M>>,
    missings: HashMap<M, (u64, u16, N, usize)>,
}
impl<N, M> MissingMessages<N, M>
where
    N: Clone,
    M: Hash + Eq + Clone,
{
    fn new() -> Self {
        MissingMessages {
            ihaves: BinaryHeap::new(),
            missings: HashMap::new(),
        }
    }

    fn push(&mut self, m: IhaveMessage<N, M>, mut expired_at: u64) {
        if !self.missings.contains_key(&m.message_id) {
            self.missings.insert(
                m.message_id.clone(),
                (expired_at, m.round, m.sender.clone(), 1),
            );
        } else {
            let entry = self.missings.get_mut(&m.message_id).expect("Never fails");
            if expired_at <= entry.0 {
                expired_at = entry.0 + 1;
            }
            entry.0 = expired_at;
            if entry.1 > m.round {
                entry.1 = m.round;
                entry.2 = m.sender.clone();
            }
            entry.3 += 1;
        }
        self.ihaves.push(MissingMessage {
            expired_at,
            message: m,
        });
    }

    fn pop_expired(&mut self, now: u64) -> Option<IhaveMessage<N, M>> {
        while self.ihaves.peek().map_or(false, |m| m.expired_at <= now) {
            let m = self.ihaves.pop().expect("Never fails");
            let delete = if let Some(e) = self.missings.get_mut(&m.message.message_id) {
                e.3 -= 1;
                e.3 == 0
            } else {
                // Already cancelled
                continue;
            };
            if delete {
                self.missings.remove(&m.message.message_id);
            }
            return Some(m.message);
        }
        None
    }

    fn cancel_timer(&mut self, message_id: &M) {
        self.missings.remove(message_id);
    }

    fn get_by_id(&self, message_id: &M) -> Option<(u16, &N)> {
        self.missings.get(message_id).map(|e| (e.1, &e.2))
    }
}

#[derive(Debug)]
struct MissingMessage<N, M> {
    expired_at: u64,
    message: IhaveMessage<N, M>,
}
impl<N, M> PartialEq for MissingMessage<N, M> {
    fn eq(&self, other: &Self) -> bool {
        self.expired_at == other.expired_at
    }
}
impl<N, M> Eq for MissingMessage<N, M> {}
impl<N, M> PartialOrd for MissingMessage<N, M> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.expired_at.partial_cmp(&self.expired_at)
    }
}
impl<N, M> Ord for MissingMessage<N, M> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.expired_at.cmp(&self.expired_at)
    }
}
