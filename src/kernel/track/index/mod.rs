mod codec;
mod bitmap;

pub(crate) use super::KernelOptions;
use std::collections::HashMap;
use codec::{Codec, Value};
use bytes::{Buf};
use super::Volume;
use anyhow::Result;

pub struct Index {
    cache: HashMap<String, (u32, u64)>,
    frees: Vec<u64>,
    volume: Volume,
    codec: Codec,
}

impl Index {
    pub fn new(volume: Volume) -> Self {
        Self {
            codec: Codec::new(),
            cache: HashMap::new(),
            frees: Vec::new(),
            volume 
        }
    }

    pub fn init(&mut self) -> Result<()> {
        let chunk = self.volume.read(0)?;
        let mut index = chunk.data.get_u64();
        let mut free_index = chunk.data.get_u64();

    loop {
        let chunk = self.volume.read(index)?;
        for value in self.codec.decoder(chunk.data,index) {
            self.cache.insert(value.2, (value.0, value.1));
        }

        if let Some(next) = chunk.next {
            index = next;
        } else {
            break;
        }
    }

    loop {
        let chunk = self.volume.read(free_index)?;
        
    }

        Ok(())
    }
}
