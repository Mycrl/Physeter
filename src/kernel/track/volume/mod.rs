pub mod codec;

pub use codec::Header;
use super::{Fs, KernelOptions};
use bytes::{BytesMut, BufMut};
use codec::{Chunk, Codec};
use anyhow::Result;
use std::rc::Rc;

/// 存储卷
///
/// 数据存储在轨道文件内，
/// 数据被拆分成固定大小的分片以链表形式写入，
/// 删除数据只会标记分片为失效，下次写入将覆盖分片
///
/// `options` 配置  
/// `free_start` 失效头索引  
/// `free_end` 失效尾部索引  
/// `codec` 编解码器 
/// `size` 轨道大小  
/// `file` 文件类  
/// `id` 轨道ID
pub struct Volume {
    options: Rc<KernelOptions>,
    free_start: u64,
    real_size: u64,
    free_end: u64,
    codec: Codec,
    pub size: u64,
    file: Fs,
    id: u16,
}

impl Volume {
    /// 创建实例
    ///
    /// ```no_run
    /// use super::{Track, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let track = Track::new(0, &options);
    /// ```
    pub fn new(id: u16, options: Rc<KernelOptions>) -> Result<Self> {
        let path = options.directory.join(format!("{}.volume", id));
        let file = Fs::new(path.as_path())?;
        Ok(Self {
            codec: Codec::new(options.clone()),
            real_size: file.stat()?.len(),
            free_start: 0,
            free_end: 0,
            size: 0,
            options,
            file,
            id,
        })
    }

    /// 获取分区头
    pub fn get_header(&mut self) -> Result<Header> {
        let mut buffer = [0u8; 32];
        self.file.promise_read(&mut buffer, 0)?;
        Ok(self.codec.decoder_header(&buffer))
    }

    /// 读取分片数据
    ///
    /// 读取单个分片数据
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Track, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut track = Track::new(0, &options);
    /// let chunk = track.read(10)?;
    /// ```
    pub fn read(&mut self, offset: u64) -> Result<Chunk> {
        let mut packet = vec![0u8; self.options.chunk_size as usize];
        self.file.promise_read(&mut packet, offset)?;
        Ok(self.codec.decoder(&packet))
    }

    /// 自由读取
    ///
    /// 这不只是读取单个分片，
    /// 而是在内部连续读取并解码分片，
    /// 不过并非一定满足给定长度，
    /// 比如没有更多数据的时候
    ///
    /// TODO:
    /// 这是一种低级读取方式，用于无法获取
    /// 索引仅靠内部链表游标前进的情况, 这
    /// 将可能导致严重的IO性能问题
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Track, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut track = Track::new(0, &options);
    /// let chunk = track.free_read(10, 100)?;
    /// ```
    pub fn free_read(&mut self, offset: u64, len: u64) -> Result<BytesMut> {
        let mut buffer = BytesMut::new();
        let mut index = offset;
        let mut size = 0;     

        // 无限循环
        // 连续读取
    loop {

        // 从磁盘中读取单个分片
        // 将分片内容附加到总长度
        let chunk = self.read(index)?;
        size += chunk.data.len() as u64;

        // 如果内部长度已溢出
        // 则排除掉溢出数据
        let append_chunk = if size > len {
            let diff_size = (size - len) as usize;
            let end_index = chunk.data.len() - diff_size;
            &chunk.data[0..end_index]
        } else {
            &chunk.data
        };

        // 将分片数据写入缓冲区
        buffer.put(append_chunk);

        // 检查下个分片游标
        // 更新内部游标
        // 如果到尾部则跳出
        if let Some(next) = chunk.next {
            index = next;
        } else {
            break;
        }

        // 总长度大于限制长度
        if size >= len {
            break;
        }
    }

        Ok(buffer)
    }

    /// 游标推进读取
    ///
    /// 按游标推进之后再读取单个分片，
    /// 用于忽略部分数据之后再读取，
    ///
    /// TODO:
    /// 这是一种低级读取方式，用于无法获取
    /// 索引仅靠内部链表游标前进的情况, 这
    /// 将可能导致严重的IO性能问题
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Track, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut track = Track::new(0, &options);
    /// let chunk = track.cursor_read(10, 10)?;
    /// ```
    pub fn cursor_read(&mut self, offset: u64, skip: u64) -> Result<Option<Chunk>> {
        let mut index = offset;
        let mut size = 0;

        // 无限循环
        // 连续推进
    loop {

        // 从磁盘中读取单个分片
        // 将分片内容附加到总长度
        let chunk = self.read(index)?;
        size += chunk.data.len() as u64;

        // 检查下个分片游标
        // 更新内部游标
        if let Some(next) = chunk.next {
            index = next;
        } else {
            return Ok(None);
        }

        // 总长度大于限制长度
        if size >= skip {
            break;
        }
    }

        Ok(Some(
            self.read(index)?
        ))
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
    ///
    /// let options = KernelOptions::default();
    /// let mut track = Track::new(0, &options);
    /// let index = track.alloc()?;
    /// ```
    pub fn alloc(&mut self) -> Result<Option<u64>> {
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
        
        // 没有失效块
        // 并且轨道不够写入
        if self.free_start == 0 {
            return Ok(None);
        }

        // 读取失效分片
        // 并解码失效分片
        let mut buffer = vec![0u8; chunk_size as usize];
        self.file.read(&mut buffer, self.free_start)?;
        let value = self.codec.decoder(&buffer);

        // 如果还有失效分片
        // 则更新链表头部为下个分片位置
        // 如果失效分片已经全部解决
        // 则归零链表头部
        let free_start = self.free_start;
        self.free_start = match value.next {
            Some(next) => next,
            None => 0,
        };

        // 归零链表头部时
        // 也同时归零链表尾部
        // 因为已无失效分片
        if self.free_start == 0 {
            self.free_end = 0
        }

        Ok(Some(free_start))
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
    ///
    /// let options = KernelOptions::default();
    /// let mut track = Track::new(0, &options);
    /// let track_id = track.remove(10)?;
    /// ```
    #[rustfmt::skip]
    pub fn remove(&mut self, index: u64) -> Result<()> {
        

        Ok(())
    }

    /// 写入分片
    ///
    /// 写入单个分片数据到磁盘文件
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Track, Chunk, KernelOptions};
    ///
    /// let chunk = Chunk {
    ///     id: 0,
    ///     exist: true,
    ///     next: Some(17),
    ///     next_track: None,
    ///     data: Bytes::from_static(b"hello"),
    /// };
    ///
    /// let options = KernelOptions::default();
    /// let mut track = Track::new(0, &options);
    /// track.write(Chunk, 20)?;
    /// ```
    pub fn write(&mut self, chunk: Chunk, index: u64) -> Result<()> {
        self.file.write(&self.codec.encoder(chunk), index)
    }
}
