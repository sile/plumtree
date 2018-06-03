use std::collections::{HashMap, HashSet};
use std::fmt;

use action::{Action, ActionQueue};
use ipc::{GossipMessage, GraftMessage, IhaveMessage, IpcMessage, PruneMessage};
use missing::MissingMessages;
use time::LogicalTime;
use Message;
use System;

/// Options for Plumtree [Node].
///
/// [Node]: ./struct.Node.html
#[derive(Debug, Clone)]
pub struct NodeOptions {
    /// Timeout duration (ticks) of a `IhaveMessage`.
    ///
    /// When the node receives a `IhaveMessage`, it sets a timer with the value of `ihave_timeout`.
    /// The timer decreases each times when `Node::tick` is called.
    /// If the timer expires before the associated `GossipMessage` is received,
    /// the node will send `GraftMessage` to the sender of the `IhaveMessage`
    /// for retrieving the payload of the message.
    ///
    /// The default value is `5`.
    pub ihave_timeout: u64,

    /// Optimization threshold.
    ///
    /// See "3.8. Optimization" of the [paper] for the description of the parameter.
    ///
    /// The default value is `2`.
    ///
    /// [paper]: http://www.gsd.inesc-id.pt/~ler/reports/srds07.pdf
    pub optimization_threshold: u16,
}
impl Default for NodeOptions {
    fn default() -> Self {
        NodeOptions {
            ihave_timeout: 5,
            optimization_threshold: 2,
        }
    }
}

/// Plumtree node.
pub struct Node<T: System> {
    id: T::NodeId,
    options: NodeOptions,
    eager_push_peers: HashSet<T::NodeId>,
    lazy_push_peers: HashSet<T::NodeId>,
    messages: HashMap<T::MessageId, T::MessagePayload>,
    missings: MissingMessages<T>,
    actions: ActionQueue<T>,
    clock: LogicalTime,
}
impl<T: System> fmt::Debug for Node<T>
where
    T::NodeId: fmt::Debug,
    T::MessageId: fmt::Debug,
    T::MessagePayload: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Node {{ id: {:?}, options: {:?}, eager_push_peers: {:?}, lazy_push_peers: {:?}, \
             messages: {:?}, missings: {:?}, actions: {:?}, clock: {:?} }}",
            self.id,
            self.options,
            self.eager_push_peers,
            self.lazy_push_peers,
            self.messages,
            self.missings,
            self.actions,
            self.clock
        )
    }
}
impl<T: System> Node<T> {
    /// Makes a new `Node` instance.
    pub fn new(node_id: T::NodeId) -> Self {
        Self::with_options(node_id, NodeOptions::default())
    }

    /// Makes a new `Node` instance with the given options.
    pub fn with_options(node_id: T::NodeId, options: NodeOptions) -> Self {
        Node {
            id: node_id,
            options,
            eager_push_peers: HashSet::new(),
            lazy_push_peers: HashSet::new(),
            messages: HashMap::new(),
            missings: MissingMessages::new(),
            actions: ActionQueue::new(),
            clock: LogicalTime(0),
        }
    }

    /// Returns the identifier of the node.
    pub fn id(&self) -> &T::NodeId {
        &self.id
    }

    /// Returns the options of the node.
    pub fn options(&self) -> &NodeOptions {
        &self.options
    }

    /// Returns the peers with which the node uses eager push gossip for diffusing application messages.
    pub fn eager_push_peers(&self) -> &HashSet<T::NodeId> {
        &self.eager_push_peers
    }

    /// Returns the peers with which the node uses lazy push gossip for diffusing application messages.
    pub fn lazy_push_peers(&self) -> &HashSet<T::NodeId> {
        &self.lazy_push_peers
    }

    /// Broadcasts the given message.
    pub fn broadcast_message(&mut self, message: Message<T>) {
        self.actions.deliver(message.clone());

        let gossip = GossipMessage::new(&self.id, message, 0);
        self.eager_push(&gossip);
        self.lazy_push(&gossip);
        self.messages
            .insert(gossip.message.id, gossip.message.payload);
    }

    /// Returns a reference to the messages kept by the node.
    pub fn messages(&self) -> &HashMap<T::MessageId, T::MessagePayload> {
        &self.messages
    }

    /// Forgets the specified message.
    ///
    /// For preventing memory shortage, this method needs to be called appropriately.
    pub fn forget_message(&mut self, message_id: &T::MessageId) {
        self.messages.remove(message_id);
    }

    /// Polls the next action that the node wants to execute.
    pub fn poll_action(&mut self) -> Option<Action<T>> {
        self.actions.pop()
    }

    /// Handles the given incoming message.
    pub fn handle_ipc_message(&mut self, message: IpcMessage<T>) {
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

    /// Accepts new neighbor.
    pub fn handle_neighbor_up(&mut self, neighbor_node_id: &T::NodeId) {
        if self.is_known_node(neighbor_node_id) || self.id == *neighbor_node_id {
            return;
        }
        for message_id in self.messages.keys() {
            let ihave = IhaveMessage::new(&self.id, message_id.clone(), 0, false);
            self.actions.send(neighbor_node_id.clone(), ihave);
        }
        self.eager_push_peers.insert(neighbor_node_id.clone());
    }

    /// Removes downed neighbor.
    pub fn handle_neighbor_down(&mut self, neighbor_node_id: &T::NodeId) {
        if !self.is_known_node(neighbor_node_id) {
            return;
        }
        self.eager_push_peers.remove(neighbor_node_id);
        self.lazy_push_peers.remove(neighbor_node_id);
    }

    /// Advances the logical time of the node by one unit.
    pub fn tick(&mut self) {
        self.clock.0 += 1;
        self.handle_expiration();
    }

    fn handle_expiration(&mut self) {
        while let Some(ihave) = self.missings.dequeue_expired(self.clock) {
            if !self.is_known_node(&ihave.sender) {
                // The node has been removed from neighbors
                continue;
            }
            self.eager_push_peers.insert(ihave.sender.clone());
            self.lazy_push_peers.remove(&ihave.sender);
            self.actions.send(
                ihave.sender,
                GraftMessage::new(&self.id, Some(ihave.message_id), ihave.round),
            );
        }
    }

    #[cfg_attr(feature = "cargo-clippy", allow(map_entry))]
    fn handle_gossip(&mut self, gossip: GossipMessage<T>) {
        if self.messages.contains_key(&gossip.message.id) {
            self.eager_push_peers.remove(&gossip.sender);
            self.lazy_push_peers.insert(gossip.sender.clone());
            self.actions
                .send(gossip.sender, PruneMessage::new(&self.id));
        } else {
            self.actions.deliver(gossip.message.clone());
            self.missings.remove(&gossip.message.id);

            self.eager_push(&gossip);
            self.lazy_push(&gossip);
            self.eager_push_peers.insert(gossip.sender.clone());
            self.lazy_push_peers.remove(&gossip.sender);
            self.optimize(&gossip);
            self.messages
                .insert(gossip.message.id, gossip.message.payload);
        }
    }

    fn handle_ihave(&mut self, ihave: IhaveMessage<T>) {
        if self.messages.contains_key(&ihave.message_id) {
            return;
        }

        let mut expiry_time = self.clock;
        if !ihave.realtime {
            expiry_time.0 += self.options.ihave_timeout;
        };
        self.missings.enqueue(ihave, expiry_time);
    }

    fn handle_graft(&mut self, mut graft: GraftMessage<T>) {
        self.eager_push_peers.insert(graft.sender.clone());
        self.lazy_push_peers.remove(&graft.sender);
        if let Some(message_id) = graft.message_id.take() {
            if let Some(payload) = self.messages.get(&message_id).cloned() {
                let gossip =
                    GossipMessage::new(&self.id, Message::new(message_id, payload), graft.round);
                self.actions.send(graft.sender, gossip);
            }
        }
    }

    fn handle_prune(&mut self, prune: PruneMessage<T>) {
        self.eager_push_peers.remove(&prune.sender);
        self.lazy_push_peers.insert(prune.sender);
    }

    fn eager_push(&mut self, gossip: &GossipMessage<T>) {
        let round = gossip.round.saturating_add(1);
        for peer in self.eager_push_peers
            .iter()
            .filter(|n| **n != gossip.sender)
        {
            let forward = GossipMessage::new(&self.id, gossip.message.clone(), round);
            self.actions.send(peer.clone(), forward);
        }
    }

    fn lazy_push(&mut self, gossip: &GossipMessage<T>) {
        let round = gossip.round.saturating_add(1);
        let ihave = IhaveMessage::new(&self.id, gossip.message.id.clone(), round, true);
        for peer in self.eager_push_peers
            .iter()
            .filter(|n| **n != gossip.sender)
        {
            self.actions.send(peer.clone(), ihave.clone());
        }
    }

    fn optimize(&mut self, gossip: &GossipMessage<T>) {
        if let Some((ihave_round, ihave_node)) =
            self.missings.get_min_round_owner(&gossip.message.id)
        {
            let optimize =
                gossip.round.checked_sub(ihave_round) >= Some(self.options.optimization_threshold);
            if optimize {
                let graft = GraftMessage::new(&self.id, None, ihave_round);
                let prune = PruneMessage::new(&self.id);
                self.actions.send(ihave_node.clone(), graft);
                self.actions.send(gossip.sender.clone(), prune);
            }
        }
    }

    fn is_known_node(&self, node_id: &T::NodeId) -> bool {
        self.eager_push_peers.contains(node_id) || self.lazy_push_peers.contains(node_id)
    }
}
