mod codec;
mod bitmap;

pub(crate) use super::KernelOptions;
use super::Volume;

pub struct Free {
    volume: Volume
}

impl Free {
    pub fn new(options: &KernelOptions, volume: Volume) -> Self {
        Self {
            volume
        }
    }

    pub fn init(&mut self) {
        
    }
}
