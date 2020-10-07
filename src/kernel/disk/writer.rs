use anyhow::Result;
use bytes::{Bytes, BytesMut};
use super::{KernelOptions, Chunk, Tracks};
use std::collections::HashSet;
use std::rc::Rc;

/// 写入回调任务
pub enum Callback {
    Index(u64),
    CreateTrack(u16),
}

/// 链表上个节点
///
/// 因为链表的特性导致写入需要延迟，
/// 所以需要保存上个节点的状态
pub struct Previous {
    id: u32,
    track: u16,
    index: u64,
    data: BytesMut
}

/// 写入流
///
/// 写入数据到轨道中，
/// 内部维护游标和写入策略
///
/// `tracks` 轨道列表  
/// `options` 配置  
/// `write_tracks` 写入轨道列表  
/// `first_track` 首个写入轨道  
/// `first_index` 首个写入索引  
/// `previous` 链表节点缓存  
/// `callback` 创建轨道回调  
/// `diff_size` 轨道最大数据长度  
/// `buffer` 内部缓冲区  
/// `track` 内部轨道索引  
/// `id` 分片索引
pub struct Writer {
    tracks: Tracks,
    pub first_track: Option<u16>,
    pub first_index: Option<u64>,
    write_tracks: HashSet<u16>,
    previous: Option<Previous>,
    buffer: BytesMut,
    diff_size: usize,
    track: u16,
    id: u32,
}

impl Writer {
    /// 创建写入流
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Writer, KernelOptions};
    /// use std::collections::HashMap;
    ///
    /// let mut tracks = HashMap::new();
    /// let options = KernelOptions::default();
    /// let writer = Writer::new(&mut tracks, options);
    /// ```
    pub fn new(tracks: Tracks, options: Rc<KernelOptions>) -> Self {
        Self {
            diff_size: (options.chunk_size - 10) as usize,
            buffer: BytesMut::new(),
            write_tracks: HashSet::new(),
            first_track: None,
            first_index: None,
            previous: None,
            track: 1,
            tracks,
            id: 0,
        }
    }

    /// 写入数据
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Writer, KernelOptions};
    /// use std::collections::HashMap;
    ///
    /// let mut tracks = HashMap::new();
    /// let options = KernelOptions::default();
    /// let mut writer = Writer::new(&mut tracks, options, async |id| {
    /// ...
    /// });
    ///
    /// writer.write(&b"hello")?;
    /// ```
    pub fn write(&mut self, chunk: Option<&[u8]>) -> Result<Option<Callback>> {
        match chunk {
            Some(data) => self.write_buffer(data, false),
            None => self.done(),
        }
    }

    /// 写入结束
    ///
    /// 当没有数据写入的时候，
    /// 将会清理写入流内部的状态，
    /// 比如检查未写入的节点以及未处理的数据
    fn done(&mut self) -> Result<Option<Callback>> {

        // 检查是否有未处理的数据
        // 如果存在未处理数据则将数据全部写入
        if self.buffer.len() > 0 {
            if let Some(callback) = self.write_buffer(&[], true)? {
                return Ok(Some(callback))
            }
        }

        // 检查是否有未处理的节点
        // 如果有未处理节点则将节点写入
        if let Some(previous) = self.previous.as_ref() {
            let mut tracks = self.tracks.borrow_mut();
            let track = tracks.get_mut(&previous.track).unwrap();
            track.write(&previous.into_chunk(None), previous.index)?;
        }

        // 遍历所有受影响的轨道
        // 为每个轨道保存状态
        for track_id in &self.write_tracks {
            let mut tracks = self.tracks.borrow_mut();
            tracks.get_mut(track_id).unwrap().flush()?;
        }

        // 回调写入结束
        // 将链表头部位置返回给上层
        if let (Some(track), Some(index)) = (self.first_track, self.first_index) {
            // return Ok(Some(Callback::FirstIndex(track, index)))
        }
        
        Ok(None)
    }

    /// 分配写入轨道
    ///
    /// 为内部分配合理的轨道游标
    #[rustfmt::skip]
    fn alloc(&mut self) -> Result<Callback> {
        let mut tracks = self.tracks.borrow_mut();
        
        // 无限循环
        // 直到匹配出可以写入的轨道
    loop {
        
        // 检查轨道是否存在
        // 如果轨道不存在通知上级创建轨道
        if !tracks.contains_key(&self.track) {
            return Ok(Callback::CreateTrack(self.track));
        }

        // 检查轨道大小是否可以写入分片
        // 如果可以则跳出，否则递加到下个轨道
        let track = tracks.get_mut(&self.track).unwrap();
        if let Some(index) = track.alloc()? {
            return Ok(Callback::Index(index));
        } else {
            self.track += 1;
            continue;
        }
    }
    }

    /// 将数据写入轨道
    ///
    /// 将数据自动分配到有空间写入的轨道上
    #[rustfmt::skip]
    fn write_buffer(&mut self, chunk: &[u8], free: bool) -> Result<Option<Callback>> {
        self.buffer.extend_from_slice(chunk);
        let diff_size = self.diff_size;

        // 无限循环
        // 直到无法继续分配
    loop {
        
        // 缓冲区为空直接跳出
        let buffer_size = self.buffer.len();
        if buffer_size == 0 {
            break;
        }

        // 检查缓冲区大小是否满足最小写入大小
        // 这里有一种情况就是完全清空，如果完全清空的时候则不检查
        if !free && buffer_size < diff_size  {
            break;
        }

        // 尝试分配轨道
        let mut index = 0;
        let alloc_result = self.alloc()?;
        if let Callback::Index(offset) = alloc_result {
            index = offset;
        } else {
            return Ok(Some(alloc_result))
        }
        
        // 为了避免不必要的重复写入
        // 所以这里先检查轨道索引是否存在
        if !self.write_tracks.contains(&self.track) {
            self.write_tracks.insert(self.track);
        }

        // 如果没有节点缓存
        // 则为首次写入，记录写入轨道和索引
        if let None = self.previous {
            self.first_track = Some(self.track);
            self.first_index = Some(index);
        }

        // 如果存在节点缓存
        // 则将节点缓存写入到轨道中
        if let Some(previous) = self.previous.as_mut() {
            let mut tracks = self.tracks.borrow_mut();
            let track = tracks.get_mut(&previous.track).unwrap();
            track.write(&previous.into_chunk(Some(index)), previous.index)?;
        }

        // 如果缓冲区大小比分配长度小
        // 则使用缓冲区大小，这里考虑一种情况就是存在
        // 尾部清理的时候，是存在不足分片大小的情况
        let off_index = std::cmp::min(
            buffer_size, 
            diff_size
        );

        // 初始化节点缓存
        self.previous = Some(Previous {
            data: self.buffer.split_to(off_index),
            track: self.track,
            id: self.id,
            index
        });

        // 分片序号递加
        self.id += 1;
    }

        Ok(None)
    }
}

impl Previous {
    #[rustfmt::skip]
    pub fn into_chunk(
        &self,
        next: Option<u64>
    ) -> Chunk {
        Chunk {
            data: Bytes::copy_from_slice(&self.data[..]),
            next,
        }
    }
}