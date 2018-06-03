use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use action::{Action, ActionQueue};
use message::{GossipMessage, GraftMessage, IhaveMessage, Message, PruneMessage};
use System;

// #[derive(Debug)]
pub struct Node<T: System> {
    node_id: T::NodeId,
    eager_push_peers: HashSet<T::NodeId>,
    lazy_push_peers: HashSet<T::NodeId>,
    missing: MissingMessages<T>,
    received_msgs: HashSet<T::MessageId>,
    action_queue: ActionQueue<T>,
    clock: u64,
}
impl<T: System> Node<T> {
    pub fn new(node_id: T::NodeId) -> Self {
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

    // TODO: pub fn broadcast();

    pub fn handle_message(&mut self, message: Message<T>) {
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

    pub fn handle_neighbour_up(&mut self, neighbour_node_id: T::NodeId) {
        if self.is_known_node(&neighbour_node_id) {
            // may be bug of peer-sampling-service
            return;
        }
        if self.node_id == neighbour_node_id {
            // TODO: metrics
            return;
        }
        self.eager_push_peers.insert(neighbour_node_id);
    }

    pub fn handle_neighbour_down(&mut self, neighbour_node_id: &T::NodeId) {
        if !self.is_known_node(neighbour_node_id) {
            // may be bug of peer-sampling-service
            return;
        }
        self.eager_push_peers.remove(neighbour_node_id);
        self.lazy_push_peers.remove(neighbour_node_id);
    }

    pub fn forget_message(&mut self, message_id: &T::MessageId) {
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

    pub fn poll_action(&mut self) -> Option<Action<T>> {
        self.action_queue.pop()
    }

    fn handle_gossip(&mut self, m: GossipMessage<T>) {
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

    fn handle_ihave(&mut self, m: IhaveMessage<T>) {
        if self.received_msgs.contains(&m.message_id) {
            return;
        }
        let expiry_time = self.clock + 3; // TODO: parameter
        self.missing.push(m, expiry_time);
    }

    fn handle_graft(&mut self, mut m: GraftMessage<T>) {
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

    fn handle_prune(&mut self, m: PruneMessage<T>) {
        self.eager_push_peers.remove(&m.sender);
        self.lazy_push_peers.insert(m.sender);
    }

    fn eager_push(&mut self, mut m: GossipMessage<T>) {
        let sender = m.sender;
        m.sender = self.node_id.clone();
        m.round = m.round.saturating_add(1);
        for p in self.eager_push_peers.iter().filter(|n| **n != sender) {
            self.action_queue.send(p.clone(), m.clone());
        }
    }

    fn lazy_push(&mut self, m: GossipMessage<T>) {
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

    fn optimize(&mut self, m: GossipMessage<T>) {
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

    fn is_known_node(&self, node_id: &T::NodeId) -> bool {
        self.eager_push_peers.contains(node_id) || self.lazy_push_peers.contains(node_id)
    }
}

// #[derive(Debug)]
struct MissingMessages<T: System> {
    ihaves: BinaryHeap<MissingMessage<T>>,
    missings: HashMap<T::MessageId, (u64, u16, T::NodeId, usize)>,
}
impl<T: System> MissingMessages<T> {
    fn new() -> Self {
        MissingMessages {
            ihaves: BinaryHeap::new(),
            missings: HashMap::new(),
        }
    }

    fn push(&mut self, m: IhaveMessage<T>, mut expired_at: u64) {
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

    fn pop_expired(&mut self, now: u64) -> Option<IhaveMessage<T>> {
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

    fn cancel_timer(&mut self, message_id: &T::MessageId) {
        self.missings.remove(message_id);
    }

    fn get_by_id(&self, message_id: &T::MessageId) -> Option<(u16, &T::NodeId)> {
        self.missings.get(message_id).map(|e| (e.1, &e.2))
    }
}

// #[derive(Debug)]
struct MissingMessage<T: System> {
    expired_at: u64,
    message: IhaveMessage<T>,
}
impl<T: System> PartialEq for MissingMessage<T> {
    fn eq(&self, other: &Self) -> bool {
        self.expired_at == other.expired_at
    }
}
impl<T: System> Eq for MissingMessage<T> {}
impl<T: System> PartialOrd for MissingMessage<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.expired_at.partial_cmp(&self.expired_at)
    }
}
impl<T: System> Ord for MissingMessage<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.expired_at.cmp(&self.expired_at)
    }
}
