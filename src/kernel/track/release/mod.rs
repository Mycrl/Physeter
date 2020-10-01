mod bitmap;

use super::KernelOptions;
use super::Volume;
use anyhow::Result;
use bytes::Buf;
use bitmap::BitMap;

/// 动态释放分区
pub struct Release {
    volume: Volume,
    diff_size: usize,
    head_size: usize,
    data_size: usize,
    index: u64
}

impl Release {
    /// 创建实例
    pub fn new(options: &KernelOptions, volume: Volume) -> Self {
        let diff_size = options.chunk_size as usize - 10;
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
        let mut buffer = [0u8; 16];
        self.volume.file.read(&mut buffer, 8)?;
        self.index = buffer.get_u64();
        Ok(())
    }

    /// 分配可写区
    pub fn alloc(&mut self, size: usize) -> Result<Vec<u64>> {
        let mut alloc_map = Vec::new();
        let mut index = self.index;

        // 无限循环
        // 遍历失效链表
    loop {

        // 获取文件数据
        // 初始化分片分配表
        let chunk = self.volume.read(index)?;
        let bitmap = BitMap::new(&chunk.data);
        let mut offset = 0;

        // 无限循环
        // 找到首个未释放索引
        // 将索引添加到分配表
    loop {
        if let Some(index) = bitmap.first_zero(self.data_size, offset) {
            offset = index;
            let u64_offset = self.head_size + index;
            alloc_map.push(u64::from_be_bytes([
                chunk.data[u64_offset],
                chunk.data[u64_offset + 1],
                chunk.data[u64_offset + 2],
                chunk.data[u64_offset + 3],
                chunk.data[u64_offset + 4],
                chunk.data[u64_offset + 5],
                chunk.data[u64_offset + 6],
                chunk.data[u64_offset + 7]
            ]));
        } else {
            break;
        }
    }
        // 检查分配表是否满足
        // 如果已满足则跳出
        if alloc_map.len() >= size {
            break;
        }

        // 检查分配表是否已经遍历到尾部
        // 如果已经到尾部则跳出
        if let Some(next) = chunk.next {
            index = next;
        } else {
            break;
        }
    }

        Ok(alloc_map)
    }

    // 释放分片
    pub fn free(&mut self, index: usize) {
        
    }
}
