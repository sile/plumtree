use std::collections::HashSet;

use action::{Action, ActionQueue};
use ipc::{GossipMessage, GraftMessage, IhaveMessage, IpcMessage, PruneMessage};
use missing::MissingMessages;
use time::LogicalTime;
use System;

// #[derive(Debug)]
pub struct Node<T: System> {
    node_id: T::NodeId,
    eager_push_peers: HashSet<T::NodeId>,
    lazy_push_peers: HashSet<T::NodeId>,
    missing: MissingMessages<T>,
    received_msgs: HashSet<T::MessageId>,
    action_queue: ActionQueue<T>,
    clock: LogicalTime,
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
            clock: LogicalTime(0),
        }
    }

    // TODO: pub fn broadcast();

    pub fn handle_message(&mut self, message: IpcMessage<T>) {
        if !self.is_known_node(message.sender()) {
            return;
        }
        match message {
            IpcMessage::Gossip(m) => self.handle_gossip(m),
            IpcMessage::Ihave(m) => self.handle_ihave(m),
            IpcMessage::Graft(m) => self.handle_graft(m),
            IpcMessage::Prune(m) => self.handle_prune(m),
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
        self.clock.0 += 1;
        while let Some(ihave) = self.missing.dequeue_expired(self.clock) {
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
        if self.received_msgs.contains(&m.message.id) {
            self.eager_push_peers.remove(&m.sender);
            self.lazy_push_peers.insert(m.sender.clone());
            self.action_queue
                .send(m.sender, PruneMessage::new(&self.node_id));
        } else {
            self.action_queue.deliver(&m);
            self.received_msgs.insert(m.message.id.clone());
            self.missing.remove(&m.message.id);

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
        let expiry_time = LogicalTime(self.clock.0 + 3); // TODO: parameter
        self.missing.enqueue(m, expiry_time);
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
                        message: unsafe { ::std::mem::uninitialized() }, // TODO
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
            message_id: m.message.id,
            round: m.round.saturating_add(1),
        };
        for p in self.eager_push_peers.iter().filter(|n| **n != sender) {
            self.action_queue.send(p.clone(), m.clone());
        }
    }

    fn optimize(&mut self, m: GossipMessage<T>) {
        if let Some((round, node)) = self.missing.get_min_round_owner(&m.message.id) {
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
