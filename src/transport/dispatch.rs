use std::collections::HashMap;
use anyhow::Result;
use super::{
    kernel::Kernel,
    Task,
    Flag
};

use tokio::sync::{
    oneshot::Sender,
    mpsc::Receiver
};

pub struct Dispatch {
    sender: Sender<Task>, 
    reader: Receiver<Task>,
    // readers: HashMap<u32>,
    kernel: Kernel
}

impl Dispatch {
    fn new(
        path: String,
        track_size: u64,
        sender: Sender<Task>, 
        reader: Receiver<Task>
    ) -> Result<Self> {
        Ok(Self {
            kernel: Kernel::new(path, track_size)?,
            // readers: HashSet::new(),
            sender,
            reader
        })
    }

    fn poll(&mut self) {
        loop {
            if let Some(task) = self.reader.blocking_recv() {
                match task {
                    Task::Begin(flag, id, key) => {
                        match flag {
                            Flag::Reader => {

                            },
                            _ => {

                            }
                        }
                    },
                    _ => {

                    }
                }
            }
        }
    }
}

/// 启动核心线程
pub fn run(
    path: String, 
    track_size: u64, 
    sender: Sender<Task>, 
    reader: Receiver<Task>
) {
    std::thread::spawn(move || {
        Dispatch::new(path, track_size, sender, reader)
            .unwrap()
            .poll()
    });
}