use super::{fs::Fs, KernelOptions};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::collections::HashMap;
use anyhow::Result;
use std::rc::Rc;

/// 惰性索引
///
/// 用于返回和缓存，
/// 因为这两项不需要多余的键
///
/// `start_chunk` 头部分片位置  
/// `start_matedata` 头部媒体数据位置
pub struct Index {
    pub start_chunk: (u16, u64),
    pub start_matedata: (u16, u64),
}

/// 索引编解码器
///
/// 用于索引的查找删除和插入
///
/// `cache` 索引缓存  
/// `buffer` 解码缓冲区  
/// `file` 文件类
pub struct Codec {
    cache: HashMap<String, Index>,
    buffer: BytesMut,
    file: Fs,
}

impl Codec {
    /// 创建编解码器
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Codec, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let codec = Codec::new(&options);
    /// ```
    pub fn new(options: Rc<KernelOptions>) -> Result<Codec> {
        let path = options.directory.join("index");
        Ok(Self {
            file: Fs::new(path.as_path())?,
            buffer: BytesMut::new(),
            cache: HashMap::new(),
        })
    }

    /// 初始化
    ///
    /// 必须对该实例调用初始化，
    /// 才能进行其他操作
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Codec, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut codec = Codec::new(&options);
    /// codec.init()?;
    /// ```
    pub fn init(&mut self) -> Result<()> {
        Ok(self.loader()?)
    }

    /// 获取索引
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Codec, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let codec = Codec::new(&options);
    /// codec.init()?;
     ///
    /// let index = codec.get("test");
    /// ```
    pub fn get(&self, name: &str) -> Option<&Index> {
        self.cache.get(name)
    }

    /// 检查索引是否存在
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Codec, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let codec = Codec::new(&options);
    /// codec.init()?;
     ///
    /// let is_exist = codec.has("test");
    /// ```
    pub fn has(&self, name: &str) -> bool {
        self.cache.contains_key(name)
    }

    /// 删除索引
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Codec, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut codec = Codec::new(&options);
    /// codec.init()?;
     ///
    /// let is_remove = codec.remove("test");
    /// ```
    pub fn remove(&mut self, name: &str) -> bool {
        self.cache.remove(name).is_some()
    }

    /// 插入索引
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Codec, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut codec = Codec::new(&options);
    /// codec.init()?;
    ///
    /// codec.set("test".to_string(), ...);
    /// ```
    pub fn set(&mut self, name: String, index: Index) -> bool {
        self.cache.insert(name, index).is_some()
    }

    /// 索引转储
    ///
    /// 将所有索引转储到磁盘文件，
    /// 注意这是必要的操作，应该在实例
    /// 关闭之前调用该方法保存状态
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Codec, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut codec = Codec::new(&options);
    /// codec.init()?;
    ///
    /// codec.set("test".to_string(), ...);
    /// codec.dump()?;
    /// ```
    pub fn dump(&mut self) -> Result<()> {
        let mut offset = 0;

        // 避免数据位冲突或者无法回收磁盘空间
        // 再落盘之前先重置为空文件
        self.file.resize(0)?;

        // 遍历所有的索引
        // 编码成缓冲区之后写入磁盘文件
        for (key, index) in self.cache.iter() {
            let packet = Self::encoder(key, index);
            self.file.write(&packet, offset)?;
            offset += packet.len() as u64;
        }

        Ok(())
    }

    /// 加载索引
    ///
    /// 从磁盘文件中扫描所有索引，
    /// 并写入内部缓存中
    #[rustfmt::skip]
    fn loader(&mut self) -> Result<()> {
        let mut offset = 0;
        
        // 无限循环
        // 按固定长度从磁盘中读取数据分片
        // 推入内部缓冲区解码
    loop {

        // 从文件中读取数据分片
        let mut buffer = [0u8; 2048];
        let size = self.file.read(&mut buffer, offset)?;
        offset += size as u64;

        // 检查读取长度
        // 如果没有数据则跳出循环
        if size == 0 {
            break;
        }

        // 解码内部缓冲区
        // 遍历返回的索引列表
        // 将索引项写入内部缓存
        for (key, value) in self.decoder(&buffer[0..size]) {
            self.cache.insert(key, value);
        }
    }

        Ok(())
    }

    /// 解码缓冲区
    ///
    /// 将缓冲区分片推入内部缓冲区
    /// 并尝试解码出所有索引
    #[rustfmt::skip]
    fn decoder(&mut self, chunk: &[u8]) -> Vec<(String, Index)> {
        self.buffer.extend_from_slice(chunk);
        let mut results = Vec::new();

        // 无限循环
        // 直到无法解码
    loop {
        
        // 检查缓冲区长度是否满足最小长度
        // 如果不满足则跳出循环
        if self.buffer.len() < 3 {
            break;
        }

        // 获取索引数据总长度
        let size = u16::from_be_bytes([
            self.buffer[0], 
            self.buffer[1]
        ]) as usize;

        // 如果缓冲区内部长度不足
        // 则跳出循环
        if size > self.buffer.len() {
            break;
        }
        
        // 内部游标前进U16
        self.buffer.advance(2);
        
        // 获取key
        // 先获取长度，然后提取key缓冲区，
        // 并以不安全的方式转为字符串
        let key_size = size - 22;
        let key_buf = self.buffer[0..key_size].to_vec();
        let key = unsafe { String::from_utf8_unchecked(key_buf) };
        self.buffer.advance(key_size);

        // 获取媒体数据头部索引
        // 获取分片头部索引
        let matedata_track = self.buffer.get_u16();
        let matedata_index = self.buffer.get_u64();
        let chunk_track = self.buffer.get_u16();
        let chunk_index = self.buffer.get_u64();

        // 将索引推入索引列表
        results.push((key, Index {
            start_matedata: (matedata_track, matedata_index),
            start_chunk: (chunk_track, chunk_index),
        }))
    }

        results
    }

    /// 编码索引
    ///
    /// 将索引编码为缓冲区
    /// 将索引写入磁盘文件时使用
    fn encoder(key: &str, index: &Index) -> Bytes {
        let mut packet = BytesMut::new();
        packet.put_u16((key.len() + 22) as u16);
        packet.put_slice(key.as_bytes());
        packet.put_u16(index.start_matedata.0);
        packet.put_u64(index.start_matedata.1);
        packet.put_u16(index.start_chunk.0);
        packet.put_u64(index.start_chunk.1);
        packet.freeze()
    }
}
