use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fmt;

use message::IhaveMessage;
use time::LogicalTime;
use System;

pub struct MissingMessages<T: System> {
    ihave_queue: BinaryHeap<WithExpiryTime<T>>,
    missings: HashMap<T::MessageId, Entry<T::NodeId>>,
}
impl<T: System> fmt::Debug for MissingMessages<T>
where
    T::NodeId: fmt::Debug,
    T::MessageId: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MissingMessages {{ ihave_queue: {:?}, missings: {:?} }}",
            self.ihave_queue, self.missings
        )
    }
}
impl<T: System> MissingMessages<T> {
    pub fn new() -> Self {
        MissingMessages {
            ihave_queue: BinaryHeap::new(),
            missings: HashMap::new(),
        }
    }

    pub fn enqueue(&mut self, ihave: IhaveMessage<T>, mut expiry_time: LogicalTime) {
        if !self.missings.contains_key(&ihave.message_id) {
            let e = Entry {
                latest_expiry_time: expiry_time,
                min_round: ihave.round,
                min_round_owner: ihave.sender.clone(),
                owner_count: 1,
            };
            self.missings.insert(ihave.message_id.clone(), e);
        } else {
            let entry = self.missings
                .get_mut(&ihave.message_id)
                .expect("Never fails");
            if expiry_time <= entry.latest_expiry_time {
                expiry_time.0 = entry.latest_expiry_time.0 + 1;
            }
            entry.latest_expiry_time = expiry_time;
            if entry.min_round > ihave.round {
                entry.min_round = ihave.round;
                entry.min_round_owner = ihave.sender.clone();
            }
            entry.owner_count += 1;
        }
        self.ihave_queue.push(WithExpiryTime { expiry_time, ihave });
    }

    pub fn dequeue_expired(&mut self, now: LogicalTime) -> Option<IhaveMessage<T>> {
        while self.ihave_queue
            .peek()
            .map_or(false, |x| x.expiry_time <= now)
        {
            let ihave = self.ihave_queue.pop().expect("Never fails").ihave;
            let empty = if let Some(e) = self.missings.get_mut(&ihave.message_id) {
                e.owner_count -= 1;
                e.owner_count == 0
            } else {
                continue;
            };
            if empty {
                self.missings.remove(&ihave.message_id);
            }
            return Some(ihave);
        }
        None
    }

    pub fn remove(&mut self, message_id: &T::MessageId) {
        self.missings.remove(message_id);
    }

    pub fn get_min_round_owner(&self, message_id: &T::MessageId) -> Option<(u16, &T::NodeId)> {
        self.missings
            .get(message_id)
            .map(|e| (e.min_round, &e.min_round_owner))
    }
}

#[derive(Debug)]
struct Entry<N> {
    latest_expiry_time: LogicalTime,
    min_round: u16,
    min_round_owner: N,
    owner_count: usize,
}

struct WithExpiryTime<T: System> {
    expiry_time: LogicalTime,
    ihave: IhaveMessage<T>,
}
impl<T: System> PartialEq for WithExpiryTime<T> {
    fn eq(&self, other: &Self) -> bool {
        self.expiry_time == other.expiry_time
    }
}
impl<T: System> Eq for WithExpiryTime<T> {}
impl<T: System> PartialOrd for WithExpiryTime<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.expiry_time.partial_cmp(&self.expiry_time)
    }
}
impl<T: System> Ord for WithExpiryTime<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.expiry_time.cmp(&self.expiry_time)
    }
}
impl<T: System> fmt::Debug for WithExpiryTime<T>
where
    T::NodeId: fmt::Debug,
    T::MessageId: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "WithExpiryTime {{ expiry_time: {:?}, ihave: {:?} }}",
            self.expiry_time, self.ihave
        )
    }
}
