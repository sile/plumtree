use std::collections::VecDeque;
use std::fmt;

use ipc::{GossipMessage, IpcMessage};
use {Message, System};

/// Actions instructed by Plumtree [Node].
///
/// For running Plumtree nodes, the actions must be handled correctly by upper layers.
///
/// [Node]: ./struct.Node.html
pub enum Action<T: System> {
    /// Send a message.
    ///
    /// If it is failed to send the message (e.g., the destination node does not exist),
    /// the message will be discarded silently.
    Send {
        /// The destination of the message.
        destination: T::NodeId,

        /// The outgoing message.
        message: IpcMessage<T>,
    },

    /// Deliver a message to the applications waiting for messages.
    Deliver {
        /// The message to be delivered.
        message: Message<T>,
    },
}
impl<T: System> Action<T> {
    pub(crate) fn send<M>(destination: T::NodeId, message: M) -> Self
    where
        M: Into<IpcMessage<T>>,
    {
        Action::Send {
            destination,
            message: message.into(),
        }
    }
}
impl<T: System> fmt::Debug for Action<T>
where
    T::NodeId: fmt::Debug,
    T::MessageId: fmt::Debug,
    T::MessagePayload: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Send {
                destination,
                message,
            } => write!(
                f,
                "Send {{ destination: {:?}, message: {:?} }}",
                destination, message
            ),
            Action::Deliver { message } => write!(f, "Deliver {{ message: {:?} }}", message),
        }
    }
}

pub struct ActionQueue<T: System>(VecDeque<Action<T>>);
impl<T: System> ActionQueue<T> {
    pub fn new() -> Self {
        ActionQueue(VecDeque::new())
    }

    pub fn send<M: Into<IpcMessage<T>>>(&mut self, destination: T::NodeId, message: M) {
        self.0.push_back(Action::send(destination, message));
    }

    pub fn deliver(&mut self, gossip: &GossipMessage<T>) {
        self.0.push_back(Action::Deliver {
            message: gossip.message.clone(),
        });
    }

    pub fn pop(&mut self) -> Option<Action<T>> {
        self.0.pop_back()
    }
}
impl<T: System> fmt::Debug for ActionQueue<T>
where
    T::NodeId: fmt::Debug,
    T::MessageId: fmt::Debug,
    T::MessagePayload: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ActionQueue({:?})", self.0)
    }
}
