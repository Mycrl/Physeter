use super::chunk::{Chunk, Codec};
use super::{fs::Fs, KernelOptions};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use anyhow::Result;
use std::rc::Rc;

/// 存储轨道
///
/// 数据存储在轨道文件内，
/// 数据被拆分成固定大小的分片以链表形式写入，
/// 删除数据只会标记分片为失效，下次写入将覆盖分片
pub struct Track {
    options: Rc<KernelOptions>,
    free_start: u64,
    real_size: u64,
    free_end: u64,
    chunk: Codec,
    size: u64,
    file: Fs,
}

impl Track {
    /// 创建轨道
    ///
    /// ```no_run
    /// use super::{Track, KernelOptions};
    /// use std::rc::Rc;
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// let track = Track::new(0, options).unwrap();
    /// ```
    pub fn new(id: u16, options: Rc<KernelOptions>) -> Result<Track> {
        let path = options.path.join(format!("{}.track", id));
        Ok(Self {
            chunk: Codec::new(options.clone()),
            file: Fs::new(path.as_path())?,
            free_start: 0,
            real_size: 0,
            free_end: 0,
            size: 0,
            options,
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
    /// use super::{Track, KernelOptions};
    /// use std::rc::Rc;
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// let mut track = Track::new(0, options).unwrap();
    /// track.init().unwrap();
    /// ```
    pub fn init(&mut self) -> Result<()> {
        self.real_size = self.file.stat()?.len();
        self.read_header()
    }

    /// 读取分片数据
    ///
    /// 读取单个分片数据
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Track, KernelOptions};
    /// use std::rc::Rc;
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// let mut track = Track::new(0, options).unwrap();
    /// track.init().unwrap();
    /// 
    /// let chunk = track.read(10).unwrap();
    /// ```
    pub fn read(&mut self, offset: u64) -> Result<Chunk> {
        let mut packet = vec![0u8; self.options.chunk_size as usize];
        self.file.promise_read(&mut packet, offset)?;
        Ok(self.chunk.decoder(Bytes::from(packet)))
    }

    /// 分配分片写入位置
    ///
    /// 因为链表的特殊性，
    /// 所以这个地方并不直接写入数据，
    /// 而是预先分配位置
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Track, KernelOptions};
    //// use std::rc::Rc;
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// let mut track = Track::new(0, options).unwrap();
    /// track.init().unwrap();
    ///
    /// let index = track.alloc().unwrap();
    /// ```
    pub fn alloc(&mut self) -> Result<Option<u64>> {
        let chunk_size = self.options.chunk_size;
        let track_size = self.options.track_size;
        let free_start = self.free_start;
        let real_size = self.real_size;

        // 避免写入放大(WAF)
        // 先写入轨道文件尾部
        if real_size + chunk_size <= track_size {
            self.real_size += chunk_size;
            self.size += chunk_size;
            return Ok(Some(real_size));
        }

        // 没有失效块
        // 并且轨道不够写入
        if free_start == 0 {
            return Ok(None);
        }

        // 读取失效分片
        // 并解码失效分片
        let mut buffer = [0u8; 8];
        self.file.read(&mut buffer, free_start)?;
        let next = u64::from_be_bytes(buffer);

        // 检查失效分片是否已经分配完成
        // 如果分配完整则重置失效分片状态
        Ok(if self.free_end > 0 && next == self.free_end {
            self.free_start = 0;
            self.free_end = 0;
            None
        } else {
            self.free_start = next;
            Some(free_start)
        })
    }

    /// 删除数据
    ///
    /// 和其他函数不同，
    /// 因为删除是个需要连续性的操作，
    /// 所以这里只用给定头部分片，
    /// 内部将一直根据链表索引删除下去，
    /// 当遇到跳出当前轨道去往其他轨道的时候，
    /// 将返回其他轨道的ID
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Track, KernelOptions};
    /// use std::rc::Rc;
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// let mut track = Track::new(0, options).unwrap();
    /// track.init().unwrap();
    ///
    /// let track_id = track.remove(10).unwrap();
    /// ```
    #[rustfmt::skip]
    pub fn remove(&mut self, alloc_map: &Vec<u64>) -> Result<()> {
        assert!(alloc_map.len() > 0);
        
        // 获取头部索引
        // 获取尾部索引
        let first = alloc_map.first().unwrap();
        let last = alloc_map.last().unwrap();
        
        // 失效索引尾部更新
        // 更新为当前尾部位置
        self.free_end = *last;
        
        // 如果当前没有已失效的块
        // 则直接更新头部索引
        // 如果存在则首尾链接
        if self.free_start > 0 {
            let next_buf = first.to_be_bytes();
            self.file.write(&next_buf, self.free_end)?;
        } else {
            self.free_start = *first;
        }
        
        // 保存状态
        self.flush()
    }

    /// 写入分片
    ///
    /// 写入单个分片数据到磁盘文件
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Track, Chunk, KernelOptions};
    /// use std::rc::Rc;
    ///
    /// let chunk = Chunk {
    ///     next: Some(17),
    ///     data: Bytes::from_static(b"hello"),
    /// };
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// let mut track = Track::new(0, options).unwrap();
    /// track.init().unwrap();
    ///
    /// track.write(&chunk, 20).unwrap();
    /// ```
    pub fn write(&mut self, chunk: &Chunk, index: u64) -> Result<()> {
        self.file.write(&self.chunk.encoder(chunk), index)
    }

    /// 写入结束
    ///
    /// 当数据流写入完成的时候，
    /// 将状态同步到磁盘文件，
    /// 这是一个必要的操作，
    /// 但是不强制什么时候调用，
    /// 不过一定要在关闭实例之前调用一次
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Track, Chunk, KernelOptions};
    /// use std::rc::Rc;
    ///
    /// let chunk = Chunk {
    ///     next: Some(17),
    ///     data: Bytes::from_static(b"hello"),
    /// };
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// let mut track = Track::new(0, options).unwrap();
    /// track.init().unwrap();
    ///
    /// track.write(Chunk, 20).unwrap();
    /// track.flush().unwrap();
    /// ```
    pub fn flush(&mut self) -> Result<()> {
        let mut packet = BytesMut::new();
        packet.put_u64(self.free_start);
        packet.put_u64(self.free_end);
        packet.put_u64(self.size);
        self.file.write(&packet, 0)?;
        self.file.flush()
    }

    /// 创建默认文件头
    ///
    /// 将默认的失效块头索引和尾部索引写入到磁盘文件,
    /// 并初始化文件长度状态
    fn default_header(&mut self) -> Result<()> {
        let mut buf = BytesMut::new();
        buf.put_u64(0);
        buf.put_u64(0);
        buf.put_u64(24);
        self.file.write(&buf, 0)?;
        self.real_size = 24;
        self.size = 24;
        Ok(())
    }

    /// 读取文件头
    ///
    /// 从磁盘文件中读取失效块头索引和尾部索引，
    /// 这是必要的操作，轨道实例化的时候必须要
    /// 从文件中恢复上次的状态
    fn read_header(&mut self) -> Result<()> {
        // 如果文件为空
        // 则直接写入默认头索引
        if self.real_size == 0 {
            return self.default_header();
        }

        // 从文件中读取头部
        let mut buffer = [0u8; 24];
        self.file.read(&mut buffer, 0)?;
        let mut packet = Bytes::from(buffer.to_vec());

        // 将状态同步到实例内部
        self.free_start = packet.get_u64();
        self.free_end = packet.get_u64();
        self.size = packet.get_u64();
        
        Ok(())
    }
}
