use crate::action::{Action, ActionQueue};
use crate::message::{
    GossipMessage, GraftMessage, IhaveMessage, Message, ProtocolMessage, PruneMessage,
};
use crate::missing::MissingMessages;
use crate::time::{Clock, NodeTime};
use crate::System;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::time::Duration;

/// Options for Plumtree [Node].
///
/// [Node]: ./struct.Node.html
#[derive(Debug, Clone)]
pub struct NodeOptions {
    /// Timeout duration of a `IhaveMessage`.
    ///
    /// When a node receives a `IhaveMessage`,
    /// the expiry time of the message is set after `ihave_timeout` duration.
    ///
    /// If it expires before the associated `GossipMessage` is received,
    /// the node will send `GraftMessage` to the sender of the `IhaveMessage`
    /// for retrieving the payload of the message.
    ///
    /// The default value is `Duration::from_millis(500)`.
    pub ihave_timeout: Duration,

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
            ihave_timeout: Duration::from_millis(500),
            optimization_threshold: 2,
        }
    }
}

/// Plumtree node.
///
/// # User's responsibility
///
/// For running a node correctly, you have to call the following methods appropriately:
///
/// - [`poll_action`]
/// - [`forget_message`]
/// - [`handle_protocol_message`]
/// - [`handle_neighbor_up`]
/// - [`handle_neighbor_down`]
/// - [`clock_mut`]
///
/// For details, refer to the document of each method.
///
/// [`poll_action`]: ./struct.Node.html#method.poll_action
/// [`forget_message`]: ./struct.Node.html#method.forget_message
/// [`handle_protocol_message`]: ./struct.Node.html#method.handle_protocol_message
/// [`handle_neighbor_up`]: ./struct.Node.html#method.handle_neighbor_up
/// [`handle_neighbor_down`]: ./struct.Node.html#method.handle_neighbor_down
/// [`clock_mut`]: ./struct.Node.html#method.clock_mut
pub struct Node<T: System> {
    id: T::NodeId,
    options: NodeOptions,
    eager_push_peers: HashSet<T::NodeId>,
    lazy_push_peers: HashSet<T::NodeId>,
    messages: HashMap<T::MessageId, T::MessagePayload>,
    missings: MissingMessages<T>,
    actions: ActionQueue<T>,
    clock: Clock,
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
            clock: Clock::new(),
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

    /// Returns a mutable reference to the options of the node.
    pub fn options_mut(&mut self) -> &mut NodeOptions {
        &mut self.options
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

    /// Returns the number of messages waiting to be received.
    ///
    /// Roughly speaking, it indicates the approximate number of `IHAVE` messages held by the node.
    pub fn waiting_messages(&self) -> usize {
        self.missings.waiting_messages()
    }

    /// Forgets the specified message.
    ///
    /// If the node does not have the target message, this method will return `false`.
    ///
    /// For preventing memory shortage, this method needs to be called appropriately.
    pub fn forget_message(&mut self, message_id: &T::MessageId) -> bool {
        self.messages.remove(message_id).is_some()
    }

    /// Polls the next action that the node wants to execute.
    pub fn poll_action(&mut self) -> Option<Action<T>> {
        self.handle_expiration();
        self.actions.pop()
    }

    /// Handles the given incoming message.
    ///
    /// This method will return `false` if the sender of the message is not a neighbor of this node.
    pub fn handle_protocol_message(&mut self, message: ProtocolMessage<T>) -> bool {
        if !self.is_known_node(message.sender()) {
            return false;
        }
        match message {
            ProtocolMessage::Gossip(m) => self.handle_gossip(m),
            ProtocolMessage::Ihave(m) => self.handle_ihave(m),
            ProtocolMessage::Graft(m) => self.handle_graft(m),
            ProtocolMessage::Prune(m) => self.handle_prune(m),
        }
        true
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

        if self.eager_push_peers.is_empty() {
            while let Some(ihave) = self.missings.pop_expired(&Clock::max()) {
                if self.send_graft(ihave) {
                    break;
                }
            }
        }
    }

    /// Returns a reference to the clock of the node.
    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    /// Returns a mutable reference to the clock of the node.
    ///
    /// Note that for handling `IHAVE` messages correctly,
    /// you have to proceed the time of the node by calling [`Clock::tick`] method.
    ///
    /// [`Clock::tick`]: ./time/struct.Clock.html#method.tick
    pub fn clock_mut(&mut self) -> &mut Clock {
        &mut self.clock
    }

    /// Returns the nearest time when the timeout of a `IHAVE` message expires.
    ///
    /// If the node has no `IHAVE` messages to be handled, this method will return `None`.
    pub fn next_expiry_time(&self) -> Option<NodeTime> {
        self.missings.next_expiry_time()
    }

    fn handle_expiration(&mut self) {
        while let Some(ihave) = self.missings.pop_expired(&self.clock) {
            self.send_graft(ihave);
        }
    }

    fn send_graft(&mut self, ihave: IhaveMessage<T>) -> bool {
        if !self.is_known_node(&ihave.sender) {
            // The node has been removed from neighbors
            false
        } else {
            self.eager_push_peers.insert(ihave.sender.clone());
            self.lazy_push_peers.remove(&ihave.sender);
            self.actions.send(
                ihave.sender,
                GraftMessage::new(&self.id, Some(ihave.message_id), ihave.round),
            );
            true
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

            self.eager_push(&gossip);
            self.lazy_push(&gossip);
            self.eager_push_peers.insert(gossip.sender.clone());
            self.lazy_push_peers.remove(&gossip.sender);

            self.optimize(&gossip);
            self.missings.remove(&gossip.message.id);
            self.messages
                .insert(gossip.message.id, gossip.message.payload);
        }
    }

    fn handle_ihave(&mut self, mut ihave: IhaveMessage<T>) {
        if self.messages.contains_key(&ihave.message_id) {
            return;
        }
        if self.eager_push_peers.is_empty() {
            ihave.realtime = true;
        }
        self.missings
            .push(ihave, &self.clock, self.options.ihave_timeout);
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
        for peer in self
            .eager_push_peers
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
        for peer in self.lazy_push_peers.iter().filter(|n| **n != gossip.sender) {
            self.actions.send(peer.clone(), ihave.clone());
        }
    }

    fn optimize(&mut self, gossip: &GossipMessage<T>) {
        if let Some((ihave_round, ihave_owner)) = self.missings.get_ihave(&gossip.message.id) {
            let optimize =
                gossip.round.checked_sub(ihave_round) >= Some(self.options.optimization_threshold);
            if optimize {
                let graft = GraftMessage::new(&self.id, None, ihave_round);
                let prune = PruneMessage::new(&self.id);
                self.actions.send(ihave_owner.clone(), graft);
                self.actions.send(gossip.sender.clone(), prune);
            }
        }
    }

    fn is_known_node(&self, node_id: &T::NodeId) -> bool {
        self.eager_push_peers.contains(node_id) || self.lazy_push_peers.contains(node_id)
    }
}
