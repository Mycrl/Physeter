use super::KernelOptions;

/// BitMap
///
/// 用于存储每个分片的空闲状态
/// 内部使用三层索引存储
/// 用于快速找到失效位
///
/// `node` 节点索引  
/// `size` bitmap大小  
/// `node_size` 节点大小
/// `buffer` 缓冲区
pub struct BitMap<'a> {
    node: (Vec<u16>, Vec<u16>, Vec<u16>),
    buffer: &'a [u8],
    node_size: u64,
    size: u64,
}

impl<'a> BitMap<'a> {
    pub fn new(options: &KernelOptions, buffer: &'a [u8]) -> Self {
        let size = options.track_size as f64 / options.chunk_size as f64;
        Self {
            node: (vec![0; 10], vec![0; 100], vec![0; 1000]),
            node_size: f64::ceil(size / 8.0) as u64,
            size: f64::ceil(size) as u64,
            buffer,
        }
    }

    fn read(&self, index: usize) {
        
    }
}
