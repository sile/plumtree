use std::collections::VecDeque;

use message::Message;
use System;

// #[derive(Debug)]
pub struct ActionQueue<T: System>(VecDeque<Action<T>>);
impl<T: System> ActionQueue<T> {
    pub fn new() -> Self {
        ActionQueue(VecDeque::new())
    }

    pub fn send<M: Into<Message<T>>>(&mut self, destination: T::NodeId, message: M) {
        self.0.push_back(Action::send(destination, message));
    }

    pub fn deliver(&mut self, message_id: T::MessageId) {
        self.0.push_back(Action::Deliver { message_id });
    }

    pub fn pop(&mut self) -> Option<Action<T>> {
        self.0.pop_back()
    }
}

// #[derive(Debug)]
pub enum Action<T: System> {
    Send {
        destination: T::NodeId,
        message: Message<T>,
    },
    Deliver {
        message_id: T::MessageId,
    },
}
impl<T: System> Action<T> {
    pub(crate) fn send<M>(destination: T::NodeId, message: M) -> Self
    where
        M: Into<Message<T>>,
    {
        Action::Send {
            destination,
            message: message.into(),
        }
    }
}
