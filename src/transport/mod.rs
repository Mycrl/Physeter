mod kernel;
mod dispatch;

use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::sync::oneshot::{
    Receiver,
    Sender
};

pub enum Flag {
    Reader,
    Writer,
    Delete
}

pub enum Task {
    Begin(Flag, u32, Arc<Bytes>),
    Payload(Flag, u32, Arc<Bytes>),
    Done(Flag, u32),
    None
}

pub struct Transport {
    senders: Arc<Mutex<HashMap<u32, Sender<Task>>>>,
    reader: Receiver<Task>
}

impl Transport {
    pub async fn register(&mut self, id: u32, stream: Sender<Task>) {
        self.senders.lock().await.entry(id).or_insert(stream);
    }
}