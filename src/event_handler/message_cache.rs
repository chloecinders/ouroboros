use std::collections::{HashMap, VecDeque};

use serenity::all::Message;

pub struct MessageCache {
    sizes: HashMap<u64, usize>,
    messages: HashMap<u64, MessageQueue>,
    inserts: HashMap<u64, usize>,
}

impl MessageCache {
    pub fn new() -> Self {
        Self {
            sizes: HashMap::new(),
            messages: HashMap::new(),
            inserts: HashMap::new(),
        }
    }

    pub fn assign_count(&mut self, channel: u64, count: usize) {
        self.sizes.insert(channel, count);
    }

    pub fn clear_inserts(&mut self) {
        self.inserts = self
            .inserts
            .clone()
            .into_keys()
            .map(|i| (i, 0_usize))
            .collect::<HashMap<_, _>>();
    }

    pub fn store_message(&mut self, channel: u64, message: Message) {
        *self.inserts.entry(channel).or_default() += 1;
        let queue_size = *self.sizes.entry(channel).or_insert(100);

        if queue_size == 0 {
            return;
        }

        let queue = self.messages.entry(channel).or_default();
        queue.insert(message);

        if queue.len() > queue_size {
            queue.pop();
        }
    }

    pub fn get_message(&mut self, channel: u64, message: u64) -> Option<&Message> {
        let queue = self.messages.entry(channel).or_default();
        queue.get(message)
    }

    // pub fn find_message(&mut self, message: u64) -> Option<&Message> {
    //     for (_, queue) in self.messages.iter() {
    //         let some_msg = queue.get(message);

    //         if some_msg.is_some() {
    //             return some_msg;
    //         }
    //     }

    //     None
    // }

    pub fn get_inserts(&self) -> HashMap<u64, usize> {
        self.inserts.clone()
    }

    pub fn get_sizes(&self) -> HashMap<u64, usize> {
        self.sizes.clone()
    }

    pub fn get_channel_len(&self, channel: u64) -> usize {
        self.messages
            .get(&channel)
            .map(|c| c.len())
            .unwrap_or_default()
    }
}

#[derive(Default)]
struct MessageQueue {
    items: VecDeque<Message>,
    index: HashMap<u64, usize>,
}

impl MessageQueue {
    fn insert(&mut self, msg: Message) {
        let id = msg.id.get();
        self.items.push_front(msg);
        self.index.insert(id, self.items.len() - 1);
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn get(&self, id: u64) -> Option<&Message> {
        self.index.get(&id).map(|&i| &self.items[i])
    }

    fn pop(&mut self) {
        let msg = self.items.pop_back();

        if let Some(msg) = msg {
            self.index.remove(&msg.id.get());
        }
    }
}
