mod codec;

use super::KernelOptions;
use super::Volume;
use anyhow::Result;

/// 动态释放分区
pub struct Release {
    volume: Volume,
    diff_size: usize,
    head_size: usize,
    data_size: usize,
    index_first: u64,
    index_end: u64
}

impl Release {
    /// 创建实例
    pub fn new(options: &KernelOptions, volume: Volume) -> Self {
        let diff_size = options.chunk_size as usize - 10;
        let head_size = diff_size / 8 / 8;
        let data_size = diff_size - head_size;
        Self {
            volume,
            diff_size,
            head_size,
            data_size,
            index_first: 0,
            index_end: 0
        }
    }

    /// 初始化
    pub fn init(&mut self) -> Result<()> {
        let header = self.volume.get_header()?;
        self.index_first = header.release_first;
        self.index_end = header.release_end;
        Ok(())
    }

    /// 分配可写区
    pub fn alloc(&mut self, size: usize) -> Result<u64> {
        let chunk_size = self.options.chunk_size as u64;
        let track_size = self.options.track_size;
        let real_size = self.real_size;
        
        // 避免写入放大(WAF)
        // 先写入轨道文件尾部
        if real_size + chunk_size <= track_size {
            self.real_size += chunk_size;
            self.size += chunk_size;
            return Ok(Some(real_size))
        }

        Ok(0)
    }

    // 释放分片
    pub fn free(&mut self, index: u64) -> Result<()> {
        

        Ok(())
    }
}
