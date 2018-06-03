use std::fmt;

use System;

/// Application message.
pub struct Message<T: System> {
    /// The identifier of the message.
    pub id: T::MessageId,

    /// The payload of the message
    pub payload: T::MessagePayload,
}
impl<T: System> Message<T> {
    /// Makes a new `Message` instance.
    ///
    /// This is equivalent to `Message { id, payload }`.
    pub fn new(id: T::MessageId, payload: T::MessagePayload) -> Self {
        Message { id, payload }
    }
}
impl<T: System> Clone for Message<T> {
    fn clone(&self) -> Self {
        Message {
            id: self.id.clone(),
            payload: self.payload.clone(),
        }
    }
}
impl<T: System> fmt::Debug for Message<T>
where
    T::MessageId: fmt::Debug,
    T::MessagePayload: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Message {{ id: {:?}, payload: {:?} }}",
            self.id, self.payload
        )
    }
}
