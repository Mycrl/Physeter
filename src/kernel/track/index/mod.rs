mod codec;

use super::Volume;
use super::KernelOptions;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use codec::{Codec, Value};
use bytes::Buf;

pub struct Index {
    cache: HashMap<String, Value>,
    frees: Vec<u64>,
    volume: Volume,
    codec: Codec,
}

impl Index {
    pub fn new(options: &KernelOptions, volume: Volume) -> Self {
        Self {
            codec: Codec::new(options),
            cache: HashMap::new(),
            frees: Vec::new(),
            volume 
        }
    }

    pub fn init(&mut self) -> Result<()> {
        self.loader()
    }

    pub fn remove(&mut self) {
        
    }

    fn loader(&mut self) -> Result<()> {
        let chunk = self.volume.read(0)?;
        let mut index = chunk.data.get_u64();
        let mut size = 0;

    loop {
        let chunk = match size == 0 {
            true => Some(self.volume.read(index)?),
            false => self.volume.read_to_cursor(index, size)?
        };

        if let None = chunk {
            break;
        }

        let value = chunk.ok_or_else(|| anyhow!("not found"))?;
        let result = self.codec.decoder(value.data, index);
        self.cache.insert(result.0, result.2);
        size = result.1 as u64;

        if let Some(next) = value.next {
            index = next;
        } else {
            break;
        }
    }

        Ok(())
    }
}
