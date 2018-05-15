use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug)]
pub struct Node<T, M>
where
    T: Hash + Eq,
    M: Hash + Eq,
{
    id: T,
    eager_push_peers: HashSet<T>,
    lazy_push_peers: HashSet<T>,
    missing: HashSet<M>,
    received_msgs: HashSet<M>,
}
impl<T, M> Node<T, M>
where
    T: Hash + Eq,
    M: Hash + Eq,
{
    pub fn new(node_id: T) -> Self {
        Node {
            id: node_id,
            eager_push_peers: HashSet::new(),
            lazy_push_peers: HashSet::new(),
            missing: HashSet::new(),
            received_msgs: HashSet::new(),
        }
    }

    // pub fn handle_gossip(&mut self, msg_id: M, round: u8,
    pub fn handle_neighbour_up(&mut self, neighbour_node_id: T) {}

    pub fn handle_neighbour_down(&mut self, neighbour_node_id: T) {}
}
