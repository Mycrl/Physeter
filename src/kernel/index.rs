use super::KernelOptions;
use bytes::{BufMut, BytesMut};
use anyhow::Result;
use rocksdb::DB;

/// 分配表
pub type AllocMap = Vec<(u16, Vec<u64>)>;

/// 索引
///
/// 索引构筑在RocksDB上，
/// 这里抽象出标准接口来
/// 操作索引存储
pub struct Index(DB);

impl Index {
    /// 创建实例
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Index, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let index = Index::new(&options).unwrap();
    /// ```
    pub fn new(options: &KernelOptions) -> Result<Self> {
        Ok(Self(DB::open_default(options.directory)?))
    }

    /// 索引是否存在
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Index, KernelOptions};
    /// use std::collections::HashMap;
    ///
    /// let options = KernelOptions::default();
    /// let mut index = Index::new(&options).unwrap();
    ///
    /// let mut alloc_map = HashMap::new();
    /// alloc_map.insert(1, vec![1, 2, 3]);
    /// 
    /// index.set(b"a", &alloc_map).unwrap();
    /// assert_eq!(index.has(b"a"), true);
    /// ```
    pub fn has(&self, key: &[u8]) -> Result<bool> {
        Ok(self.0.get_pinned(key)?.is_some())
    }

    /// 索引是否存在
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Index, KernelOptions};
    /// use std::collections::HashMap;
    ///
    /// let options = KernelOptions::default();
    /// let mut index = Index::new(&options).unwrap();
    ///
    /// let mut alloc_map = HashMap::new();
    /// alloc_map.insert(1, vec![1, 2, 3]);
    /// 
    /// index.set(b"a", &alloc_map).unwrap();
    /// assert_eq!(index.has(b"a").unwrap(), true);
    ///
    /// index.remove(b"a").unwrap();
    /// assert_eq!(index.has(b"a").unwrap(), false);
    /// ```
    pub fn remove(&mut self, key: &[u8]) -> Result<()> {
        self.0.delete(key)?;
        Ok(())
    }

    /// 索引是否存在
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Index, KernelOptions};
    /// use std::collections::HashMap;
    ///
    /// let options = KernelOptions::default();
    /// let mut index = Index::new(&options).unwrap();
    ///
    /// let mut alloc_map = HashMap::new();
    /// alloc_map.insert(1, vec![1, 2, 3]);
    /// 
    /// index.set(b"a", &alloc_map).unwrap();
    /// 
    /// if let Some(value) = index.get(b"test").unwrap().get_mut(&1) {
    ///     assert_eq!(value.next(), Some(1));
    ///     assert_eq!(value.next(), Some(2));
    ///     assert_eq!(value.next(), Some(3));
    ///     assert_eq!(value.next(), None);
    /// }
    /// 
    /// ```
    #[rustfmt::skip]
    pub fn get(&self, key: &[u8]) -> Result<Option<AllocMap>> {
        Ok(match self.0.get_pinned(key)? {
            Some(x) => Some(decoder(unsafe { 
                std::mem::transmute(&*x) 
            })), None => None
        })
    }

    /// 写入索引项
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Index, KernelOptions};
    /// use std::collections::HashMap;
    ///
    /// let options = KernelOptions::default();
    /// let mut index = Index::new(&options).unwrap();
    ///
    /// let mut alloc_map = HashMap::new();
    /// alloc_map.insert(1, vec![1, 2, 3]);
    /// 
    /// index.set(b"a", &alloc_map).unwrap();
    /// assert_eq!(index.has(b"a").unwrap(), true);
    /// ```
    pub fn set(&mut self, key: &[u8], value: &AllocMap) -> Result<()> {
        self.0.put(key, &encoder(value)[..])?;
        Ok(())
    }
}

/// 解码索引
///
/// 将索引缓冲区转为
/// 可迭代的索引列表
#[rustfmt::skip]
fn decoder(chunk: &[u8]) -> AllocMap {
    let count_size = chunk.len();
    let mut result = Vec::new();
    let mut index = 0;

    // 无限循环
    // 迭代所有轨道
loop {
    if index >= count_size {
        break;
    }

    // 轨道ID
    let id = u16::from_be_bytes([
        chunk[index],
        chunk[index + 1]
    ]);

    // 索引列表长度
    let item_size = u32::from_be_bytes([
        chunk[index + 2],
        chunk[index + 3],
        chunk[index + 4],
        chunk[index + 5]
    ]) as usize;

    // 索引列表真实长度
    // 检查索引列表是否足够解码
    let size = (item_size * 8) + 6;
    if index + size > count_size {
        break;
    }

    // 获取区间分片
    // 创建迭代器并推入轨道列表
    let start_index = index + 6;
    let end_index = index + size;
    let chunk_slice = &chunk[start_index..end_index];
    let mut list = Vec::new();
    
    for i in 0..item_size {
        let index = i * 8;
        list.push(u64::from_be_bytes([
            chunk_slice[index],
            chunk_slice[index + 1],
            chunk_slice[index + 2],
            chunk_slice[index + 3],
            chunk_slice[index + 4],
            chunk_slice[index + 5],
            chunk_slice[index + 6],
            chunk_slice[index + 7]
        ]));
    }

    index += size;
    result.push((
        id, list
    ));
}

    result
}

/// 编码索引
///
/// 将索引分配表转为
/// 字节缓冲区
fn encoder(map: &AllocMap) -> BytesMut {
    let mut packet = BytesMut::new();
    for (id, value) in map {
        packet.put_u16(*id);
        packet.put_u32(value.len() as u32);
        for index in value {
            packet.put_u64(*index);
        }
    }

    packet
}