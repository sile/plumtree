use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fmt;
use std::time::Duration;

use message::IhaveMessage;
use time::{Clock, NodeTime};
use System;

pub struct MissingMessages<T: System> {
    ihave_queue: BinaryHeap<QueueItem<T>>, // TODO: rename
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

    pub fn enqueue(&mut self, ihave: IhaveMessage<T>, clock: &Clock, timeout: Duration) {
        let expiry_time = if !self.missings.contains_key(&ihave.message_id) {
            let mut expiry_time = clock.now();
            if !ihave.realtime {
                expiry_time += timeout;
            }

            let e = Entry {
                min_round: ihave.round,
                min_round_owner: ihave.sender.clone(),
                owners: 1,
                entry_expiry_time: clock.now() + timeout,
            };
            self.missings.insert(ihave.message_id.clone(), e);

            expiry_time
        } else {
            let entry = self.missings
                .get_mut(&ihave.message_id)
                .expect("never fails");

            let expiry_time = entry.entry_expiry_time;
            entry.entry_expiry_time += timeout;
            if entry.min_round > ihave.round {
                // TODO:
                entry.min_round = ihave.round;
                entry.min_round_owner = ihave.sender.clone();
            }
            entry.owners += 1;

            expiry_time
        };
        self.ihave_queue
            .push(QueueItem::Ihave { expiry_time, ihave });
    }

    pub fn dequeue_expired(&mut self, clock: &Clock) -> Option<IhaveMessage<T>> {
        let is_expired = |x: &QueueItem<_>| x.expiry_time() <= clock.now();
        while self.ihave_queue.peek().map_or(false, is_expired) {
            let item = self.ihave_queue.pop().expect("never fails");
            if !self.missings.contains_key(item.message_id()) {
                continue;
            }

            match item {
                QueueItem::Ihave { ihave, .. } => {
                    let entry = self.missings
                        .get_mut(&ihave.message_id)
                        .expect("never fails");
                    assert_ne!(entry.owners, 0);
                    entry.owners -= 1;
                    if entry.owners == 0 {
                        let expiry_time = entry.entry_expiry_time;
                        let message_id = ihave.message_id.clone();
                        self.ihave_queue.push(QueueItem::Entry {
                            expiry_time,
                            message_id,
                        });
                    }
                    return Some(ihave);
                }
                QueueItem::Entry { message_id, .. } => {
                    let expired = self.missings.get(&message_id).map(|e| e.owners) == Some(0);
                    if expired {
                        self.missings.remove(&message_id);
                    }
                }
            }
        }
        None
    }

    pub fn next_time(&self) -> Option<NodeTime> {
        self.ihave_queue.peek().map(|x| x.expiry_time())
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
    min_round: u16, // TODO: s/min/next/
    min_round_owner: N,
    owners: usize,
    entry_expiry_time: NodeTime,
}

enum QueueItem<T: System> {
    Ihave {
        expiry_time: NodeTime,
        ihave: IhaveMessage<T>,
    },
    Entry {
        expiry_time: NodeTime,
        message_id: T::MessageId,
    },
}
impl<T: System> QueueItem<T> {
    fn expiry_time(&self) -> NodeTime {
        match self {
            QueueItem::Ihave { expiry_time, .. } | QueueItem::Entry { expiry_time, .. } => {
                *expiry_time
            }
        }
    }

    fn message_id(&self) -> &T::MessageId {
        match self {
            QueueItem::Ihave { ihave, .. } => &ihave.message_id,
            QueueItem::Entry { message_id, .. } => message_id,
        }
    }
}
impl<T: System> PartialEq for QueueItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.expiry_time() == other.expiry_time()
    }
}
impl<T: System> Eq for QueueItem<T> {}
impl<T: System> PartialOrd for QueueItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.expiry_time().partial_cmp(&self.expiry_time())
    }
}
impl<T: System> Ord for QueueItem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.expiry_time().cmp(&self.expiry_time())
    }
}
impl<T: System> fmt::Debug for QueueItem<T>
where
    T::NodeId: fmt::Debug,
    T::MessageId: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "QueueItem {{ expiry_time: {:?}, .. }}",
            self.expiry_time()
        )
    }
}
