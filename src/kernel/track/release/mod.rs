mod bitmap;

use super::KernelOptions;
use super::Volume;
use anyhow::Result;
use bytes::Buf;
use bitmap::BitMap;

/// 动态释放分区
pub struct Release {
    volume: Volume,
    diff_size: u16,
    head_size: u16,
    data_size: u16,
    index: u64
}

impl Release {
    /// 创建实例
    pub fn new(options: &KernelOptions, volume: Volume) -> Self {
        let diff_size = options.chunk_size - 10;
        let head_size = diff_size / 8 / 8;
        let data_size = diff_size - head_size;
        Self {
            index: 0,
            diff_size,
            head_size,
            data_size,
            volume
        }
    }

    /// 初始化
    pub fn init(&mut self) -> Result<()> {
        let chunk = self.volume.free_read(8, 8)?;
        self.index = chunk.get_u64();
        Ok(())
    }

    /// 分配可写区
    pub fn alloc(&mut self, size: usize) -> Result<Vec<u64>> {
        let alloc_map = Vec::new();

        loop {
            let mut chunk = self.volume.read(self.index)?;
            let bitmap = BitMap::new(&mut chunk.data);
        }

        Ok(alloc_map)
    }
}
