use std::collections::VecDeque;

use message::Message;

#[derive(Debug)]
pub struct ActionQueue<N, M>(VecDeque<Action<N, M>>);
impl<N, M> ActionQueue<N, M> {
    pub fn new() -> Self {
        ActionQueue(VecDeque::new())
    }

    pub fn send<T: Into<Message<N, M>>>(&mut self, destination: N, message: T) {
        self.0.push_back(Action::send(destination, message));
    }

    pub fn deliver(&mut self, message_id: M) {
        self.0.push_back(Action::Deliver { message_id });
    }

    pub fn pop(&mut self) -> Option<Action<N, M>> {
        self.0.pop_back()
    }
}

#[derive(Debug)]
pub enum Action<N, M> {
    Send {
        destination: N,
        message: Message<N, M>,
    },
    Deliver {
        message_id: M,
    },
}
impl<N, M> Action<N, M> {
    pub(crate) fn send<T>(destination: N, message: T) -> Self
    where
        T: Into<Message<N, M>>,
    {
        Action::Send {
            destination,
            message: message.into(),
        }
    }
}
