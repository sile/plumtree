//! A Rust implementation of [Plumtree] algorithm.
//!
//! # References
//!
//! - [Plumtree: Epidemic Broadcast Trees][Plumtree]
//!
//! [Plumtree]: http://www.gsd.inesc-id.pt/~ler/reports/srds07.pdf
#![warn(missing_docs)]
pub use action::Action;
pub use node::{Node, NodeOptions};
pub use system::System;

mod action;
mod missing;
mod node;
mod system;

pub mod message;
pub mod time;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use message::Message;

    struct TestSystem;
    impl System for TestSystem {
        type NodeId = String;
        type MessageId = u64;
        type MessagePayload = ();
    }

    #[test]
    fn single_node_works() {
        let mut node = Node::<TestSystem>::new("foo".to_owned());
        assert_eq!(node.eager_push_peers().len(), 0);
        assert_eq!(node.lazy_push_peers().len(), 0);
        assert_eq!(node.messages().len(), 0);
        assert_eq!(node.waiting_messages(), 0);
        assert_eq!(node.clock().now().as_duration(), Duration::from_secs(0));
        assert!(node.poll_action().is_none());

        node.broadcast_message(message(0));

        let delivered = execute_single(&mut node);
        assert_eq!(delivered, vec![message(0)]);
        assert_eq!(node.messages().len(), 1);
        assert_eq!(node.waiting_messages(), 0);

        node.forget_message(&0);
        assert_eq!(node.messages().len(), 0);
    }

    #[test]
    fn multi_node_works() {
        let mut nodes: Vec<Node<TestSystem>> = vec![
            Node::new("foo".to_owned()),
            Node::new("bar".to_owned()),
            Node::new("baz".to_owned()),
            Node::new("qux".to_owned()),
        ];

        // setup neighbors
        for edges in &[
            ("foo".to_owned(), "bar".to_owned()),
            ("foo".to_owned(), "qux".to_owned()),
            ("bar".to_owned(), "baz".to_owned()),
            ("bar".to_owned(), "qux".to_owned()),
        ][..]
        {
            get(&mut nodes, &edges.0).handle_neighbor_up(&edges.1);
            get(&mut nodes, &edges.1).handle_neighbor_up(&edges.0);
        }
        assert_eq!(nodes[0].eager_push_peers().len(), 2);
        assert_eq!(nodes[1].eager_push_peers().len(), 3);
        assert_eq!(nodes[2].eager_push_peers().len(), 1);
        assert_eq!(nodes[3].eager_push_peers().len(), 2);

        // brodacast a message
        nodes[0].broadcast_message(message(0));
        execute(&mut nodes);
        for node in &nodes {
            assert_eq!(node.messages().len(), 1);
            assert_eq!(node.messages().get(&0), Some(&()));
            assert_eq!(node.waiting_messages(), 0);
        }
    }

    #[test]
    fn many_node_works() {
        let mut nodes: Vec<Node<TestSystem>> = (0..500).map(|i| Node::new(i.to_string())).collect();

        // setup neighbors
        for i in 0..nodes.len() {
            let neighbors = rand::random::<u32>() % 3 + 1;
            for _ in 0..neighbors {
                let j = rand::random::<u32>() as usize % nodes.len();
                nodes[i].handle_neighbor_up(&j.to_string());
                nodes[j].handle_neighbor_up(&i.to_string());
            }
        }

        // broadcast messages
        const MESSAGE_COUNT: usize = 50;
        for m in 0..MESSAGE_COUNT {
            let sender = rand::random::<u32>() as usize % nodes.len();
            nodes[sender].broadcast_message(message(m as u64));
        }

        execute(&mut nodes);
        for node in &nodes {
            assert_eq!(node.messages().len(), MESSAGE_COUNT);
            assert_eq!(node.waiting_messages(), 0);
        }
    }

    fn message(id: u64) -> Message<TestSystem> {
        Message { id, payload: () }
    }

    fn execute_single(node: &mut Node<TestSystem>) -> Vec<Message<TestSystem>> {
        let mut delivered = Vec::new();
        while let Some(action) = node.poll_action() {
            match action {
                Action::Deliver { message } => {
                    delivered.push(message);
                }
                Action::Send { .. } => panic!("{:?}", action),
            }
        }
        delivered
    }

    fn get<'a>(nodes: &'a mut [Node<TestSystem>], id: &String) -> &'a mut Node<TestSystem> {
        nodes.iter_mut().find(|n| n.id() == id).unwrap()
    }

    fn execute(nodes: &mut [Node<TestSystem>]) {
        let mut did_something = true;
        while did_something {
            did_something = false;

            let mut i = 0;
            while i < nodes.len() {
                while let Some(action) = nodes[i].poll_action() {
                    did_something = true;
                    match action {
                        Action::Deliver { .. } => {}
                        Action::Send {
                            destination,
                            message,
                        } => {
                            get(nodes, &destination).handle_protocol_message(message);
                        }
                    }
                }
                i += 1;
            }
            for node in nodes.iter_mut() {
                node.clock_mut().tick(Duration::from_millis(100));
            }
        }
    }
}
