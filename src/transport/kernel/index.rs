use super::KernelOptions;
use std::path::Path;
use anyhow::Result;
use rocksdb::DB;
use bytes::{
    Buf, 
    BufMut, 
    BytesMut
};

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
    /// use std::rc::Rc;
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// let index = Index::new(options).unwrap();
    /// ```
    pub fn new(options: &KernelOptions) -> Result<Self> {
        let path: &Path = &options.path.as_ref();
        let index_path = path.join("index");
        Ok(Self(DB::open_default(index_path)?))
    }

    /// 索引是否存在
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Index, KernelOptions};
    /// use std::collections::HashMap;
    /// use std::rc::Rc;
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// let mut index = Index::new(options).unwrap();
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

    /// 删除索引
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Index, KernelOptions};
    /// use std::collections::HashMap;
    /// use std::rc::Rc;
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// let mut index = Index::new(options).unwrap();
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

    /// 获取索引
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Index, KernelOptions};
    /// use std::collections::HashMap;
    /// use std::rc::Rc;
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// let mut index = Index::new(options).unwrap();
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
            Some(x) => Some(decoder(x.as_ref())), 
            None => None
        })
    }

    /// 写入索引项
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Index, KernelOptions};
    /// use std::collections::HashMap;
    /// use std::rc::Rc;
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// let mut index = Index::new(options).unwrap();
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
fn decoder(mut chunk: &[u8]) -> AllocMap {
    let mut result = Vec::new();

    // 无限循环
    // 迭代所有轨道
loop {
    if chunk.len() < 8 {
        break;
    }

    // 轨道ID
    // 索引列表长度
    let id = chunk.get_u16();
    let item_size = chunk.get_u32() as usize;

    // 索引列表真实长度
    // 检查索引列表是否足够解码
    if item_size * 8 > chunk.len() {
        break;
    }
    
    // 读取索引列表
    let mut list = Vec::new();
    for _ in 0..item_size {
        list.push(chunk.get_u64());
    }

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
