use super::KernelOptions;
use super::bitmap::BitMap;

pub struct Codec {
    diff_size: u16,
    head_size: u16,
    data_size: u16
}

impl Codec {
    pub fn new(options: &KernelOptions) -> Self {
        let diff_size = options.chunk_size - 10;
        let head_size = diff_size / 8 / 8;
        let data_size = diff_size - head_size;
        Self {
            diff_size,
            head_size,
            data_size
        }
    }

    pub fn decoder(&self, chunk: &mut [u8]) {
        let bitmap = BitMap::new(chunk);
    }
}
