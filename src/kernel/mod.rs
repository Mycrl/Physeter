mod chunk;
mod disk;
pub mod fs;
mod index;
mod track;

use std::{path::Path, rc::Rc};
use disk::{Disk, Reader, Writer};
use index::{Codec, Index};
use anyhow::Result;

pub struct KernelOptions {
    pub directory: &'static Path,
    pub track_size: u64,
    pub chunk_size: u64,
    pub max_memory: u64,
}

pub struct Kernel {
    index: Codec,
    disk: Disk
}

impl Kernel {
    pub fn new(options: KernelOptions) -> Result<Self> {
        let configure = Rc::new(options);
        Ok(Self {
            index: Codec::new(configure.clone())?,
            disk: Disk::new(configure.clone())
        })
    }

    pub fn open(&mut self) -> Result<()> {
        self.index.init()?;
        self.disk.init()?;
        Ok(())
    }

    pub fn read(&mut self, name: impl ToString) -> Result<Option<Reader>> {
        Ok(match self.index.get(&name.to_string()) {
            Some(Index { start_chunk, .. }) => {
                Some(self.disk.read(start_chunk.0, start_chunk.1))
            }, None => None
        })
    }

    pub fn write(&mut self, name: impl ToString) -> Result<Option<Writer<dyn FnMut(u16) -> Result<()> + '_>>> {
        Ok(match self.index.get(&name.to_string()) {
            None => Some(self.disk.write()),
            Some(_) => None, 
        })
    }

    pub fn delete(&mut self, name: impl ToString) -> bool {
        self.index.remove(&name.to_string())
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.index.dump()
    }
}

impl<'a> Default for KernelOptions {
    fn default() -> Self {
        Self {
            directory: Path::new("./"),
            track_size: 1024 * 1024 * 1024 * 50,
            max_memory: 1024 * 1024 * 1024,
            chunk_size: 1024 * 4,
        }
    }
}
