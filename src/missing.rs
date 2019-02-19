use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fmt;
use std::time::Duration;

use message::IhaveMessage;
use time::{Clock, NodeTime};
use System;

pub struct MissingMessages<T: System> {
    timeout_queue: BinaryHeap<QueueItem<T>>,
    ihaves: HashMap<T::MessageId, IhaveEntry<T::NodeId>>,
    entry_seqno: u64,
}
impl<T: System> fmt::Debug for MissingMessages<T>
where
    T::NodeId: fmt::Debug,
    T::MessageId: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MissingMessages {{ timeout_queue: {:?}, ihaves: {:?}, entry_seqno: {:?} }}",
            self.timeout_queue, self.ihaves, self.entry_seqno
        )
    }
}
impl<T: System> MissingMessages<T> {
    pub fn new() -> Self {
        MissingMessages {
            timeout_queue: BinaryHeap::new(),
            ihaves: HashMap::new(),
            entry_seqno: 0,
        }
    }

    pub fn push(&mut self, ihave: IhaveMessage<T>, clock: &Clock, timeout: Duration) {
        let seqno = self.entry_seqno;
        let entry = self
            .ihaves
            .entry(ihave.message_id.clone())
            .or_insert_with(|| {
                let mut expiry_time = clock.now();
                if !ihave.realtime {
                    expiry_time += timeout;
                }
                IhaveEntry {
                    seqno,
                    head_round: ihave.round,
                    head_owner: ihave.sender.clone(),
                    owners: 0,
                    next_expiry_time: expiry_time,
                }
            });

        let expiry_time = entry.next_expiry_time;
        entry.next_expiry_time += timeout;
        entry.owners += 1;
        if entry.owners == 1 {
            self.entry_seqno += 1;
        }

        self.timeout_queue.push(QueueItem::Message {
            expiry_time,
            ihave,
            entry_seqno: entry.seqno,
        });
    }

    pub fn pop_expired(&mut self, clock: &Clock) -> Option<IhaveMessage<T>> {
        let is_expired = |x: &QueueItem<_>| x.expiry_time() <= clock.now();
        while self.timeout_queue.peek().map_or(false, is_expired) {
            let item = self.timeout_queue.pop().expect("never fails");
            match self.ihaves.get(item.message_id()) {
                None => {
                    // (a) The entry has been removed due to reception of the associated GOSSIP message
                    continue;
                }
                Some(entry) if entry.seqno != item.entry_seqno() => {
                    // (b) Like `(a)`, but the message has been forgot before receiving new IHAVE messages
                    continue;
                }
                _ => {}
            }

            match item {
                QueueItem::Message { ihave, .. } => {
                    let entry = self.ihaves.get_mut(&ihave.message_id).expect("never fails");
                    assert_ne!(entry.owners, 0);

                    entry.owners -= 1;
                    entry.head_round = ihave.round;
                    entry.head_owner = ihave.sender.clone();
                    if entry.owners == 0 {
                        self.timeout_queue.push(QueueItem::Entry {
                            expiry_time: entry.next_expiry_time,
                            entry_seqno: entry.seqno,
                            message_id: ihave.message_id.clone(),
                        });
                    }
                    return Some(ihave);
                }
                QueueItem::Entry { message_id, .. } => {
                    let expired = self.ihaves.get(&message_id).map(|e| e.owners) == Some(0);
                    if expired {
                        self.ihaves.remove(&message_id);
                    }
                }
            }
        }
        None
    }

    pub fn remove(&mut self, message_id: &T::MessageId) {
        self.ihaves.remove(message_id);
    }

    pub fn waiting_messages(&self) -> usize {
        self.ihaves.len()
    }

    pub fn next_expiry_time(&self) -> Option<NodeTime> {
        self.timeout_queue.peek().map(|x| x.expiry_time())
    }

    pub fn get_ihave(&self, message_id: &T::MessageId) -> Option<(u16, &T::NodeId)> {
        self.ihaves
            .get(message_id)
            .map(|e| (e.head_round, &e.head_owner))
    }
}

#[derive(Debug)]
struct IhaveEntry<N> {
    seqno: u64,
    head_round: u16,
    head_owner: N,
    owners: usize,
    next_expiry_time: NodeTime,
}

enum QueueItem<T: System> {
    Message {
        expiry_time: NodeTime,
        entry_seqno: u64,
        ihave: IhaveMessage<T>,
    },
    Entry {
        expiry_time: NodeTime,
        entry_seqno: u64,
        message_id: T::MessageId,
    },
}
impl<T: System> QueueItem<T> {
    fn expiry_time(&self) -> NodeTime {
        match self {
            QueueItem::Message { expiry_time, .. } | QueueItem::Entry { expiry_time, .. } => {
                *expiry_time
            }
        }
    }

    fn entry_seqno(&self) -> u64 {
        match self {
            QueueItem::Message { entry_seqno, .. } | QueueItem::Entry { entry_seqno, .. } => {
                *entry_seqno
            }
        }
    }

    fn message_id(&self) -> &T::MessageId {
        match self {
            QueueItem::Message { ihave, .. } => &ihave.message_id,
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
            "QueueItem {{ expiry_time: {:?}, message_id: {:?}, .. }}",
            self.expiry_time(),
            self.message_id()
        )
    }
}
