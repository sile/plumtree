use std::collections::HashSet;
use std::hash::Hash;

use message::Message;

#[derive(Debug)]
pub struct Node<N, M>
where
    N: Hash + Eq,
    M: Hash + Eq,
{
    id: N,
    eager_push_peers: HashSet<N>,
    lazy_push_peers: HashSet<N>,
    missing: HashSet<M>,
    received_msgs: HashSet<M>,
}
impl<N, M> Node<N, M>
where
    N: Hash + Eq,
    M: Hash + Eq,
{
    pub fn new(node_id: N) -> Self {
        Node {
            id: node_id,
            eager_push_peers: HashSet::new(),
            lazy_push_peers: HashSet::new(),
            missing: HashSet::new(),
            received_msgs: HashSet::new(),
        }
    }

    pub fn handle_message(&mut self, message: Message<N, M>) {}

    pub fn handle_neighbour_up(&mut self, neighbour_node_id: N) {}

    pub fn handle_neighbour_down(&mut self, neighbour_node_id: N) {}
}
