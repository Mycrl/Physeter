use anyhow::Result;
use bytes::BytesMut;
use super::{KernelOptions, Chunk, Tracks};
use std::collections::HashSet;
use std::future::Future;

/// 链表上个节点
///
/// 因为链表的特性导致写入需要延迟，
/// 所以需要保存上个节点的状态
pub struct Previous {
    id: u32,
    track: u16,
    index: u64,
    data: BytesMut,
    next: Option<u64>,
    next_track: Option<u16>,
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
pub struct Writer<'a, R> {
    tracks: &'a mut Tracks<'a>,
    options: &'a KernelOptions<'a>,
    pub first_track: Option<u16>,
    pub first_index: Option<u64>,
    write_tracks: HashSet<u16>,
    previous: Option<Previous>,
    callback: fn(u16) -> R,
    diff_size: u64,
    buffer: BytesMut,
    track: u16,
    id: u32,
}

impl<'a, R: Future<Output = Result<()>>> Writer<'a, R>{
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
    /// let writer = Writer::new(&mut tracks, options, async |id| {
    /// ...
    /// });
    /// ```
    pub fn new(
        tracks: &'a mut Tracks<'a>,
        options: &'a KernelOptions<'_>,
        callback: fn(u16) -> R,
    ) -> Self {
        Self {
            diff_size: options.chunk_size - 17,
            buffer: BytesMut::new(),
            write_tracks: HashSet::new(),
            first_track: None,
            first_index: None,
            previous: None,
            callback,
            options,
            tracks,
            track: 0,
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
    /// writer.write(&b"hello").await?;
    /// ```
    pub async fn write(&mut self, chunk: Option<&[u8]>) -> Result<()> {
        match chunk {
            Some(data) => self.write_buffer(data, false).await,
            None => self.done().await,
        }
    }

    /// 写入结束
    ///
    /// 当没有数据写入的时候，
    /// 将会清理写入流内部的状态，
    /// 比如检查未写入的节点以及未处理的数据
    pub async fn done(&mut self) -> Result<()> {

        // 检查是否有未处理的数据
        // 如果存在未处理数据则将数据全部写入
        if self.buffer.len() > 0 {
            self.write_buffer(&[0], true).await?;
        }

        // 检查是否有未处理的节点
        // 如果有未处理节点则将节点写入
        if let Some(previous) = self.previous.as_ref() {
            let track = self.tracks.get_mut(&previous.track).unwrap();
            track.write(previous.into_chunk(), previous.index).await?;
        }

        // 遍历所有受影响的轨道
        // 为每个轨道保存状态
        for track_id in &self.write_tracks {
            self.tracks.get_mut(track_id).unwrap().write_end().await?;
        }

        Ok(())
    }

    /// 分配写入轨道
    ///
    /// 为内部分配合理的轨道游标
    async fn alloc(&mut self) -> Result<()> {
        
        // 无限循环
        // 直到匹配出可以写入的轨道
    loop {
        
        // 检查轨道是否存在
        // 如果轨道不存在通知上级创建轨道
        if !self.tracks.contains_key(&self.track) {
            (self.callback)(self.track).await?;
            break;
        }

        // 检查轨道大小是否可以写入分片
        // 如果可以则跳出，否则递加到下个轨道
        let track = self.tracks.get(&self.track).unwrap();
        if track.size + self.options.chunk_size > self.options.track_size {
            self.track += 1;
            continue;
        } else {
            break;
        }
    }

        Ok(())
    }

    /// 将数据写入轨道
    ///
    /// 将数据自动分配到有空间写入的轨道上
    async fn write_buffer(&mut self, chunk: &[u8], free: bool) -> Result<()> {
        self.buffer.extend_from_slice(chunk);
        let diff_size = self.diff_size as usize;

        // 无限循环
        // 直到无法继续分配
    loop {

        // 检查缓冲区大小是否满足最小写入大小
        // 这里有一种情况就是完全清空，如果完全清空的时候则不检查
        if !free && self.buffer.len() < diff_size  {
            break;
        }

        // 尝试分配轨道
        self.alloc().await?;

        // 为了避免不必要的重复写入
        // 所以这里先检查轨道索引是否存在
        if !self.write_tracks.contains(&self.track) {
            self.write_tracks.insert(self.track);
        }

        // 为当前轨道分配索引
        let current_track = self.tracks.get_mut(&self.track).unwrap();
        let index = current_track.alloc().await?;

        // 如果没有节点缓存
        // 则为首次写入，记录写入轨道和索引
        if let None = self.previous {
            self.first_track = Some(self.track);
            self.first_index = Some(index);
        }

        // 如果存在节点缓存
        // 则将节点缓存写入到轨道中
        if let Some(previous) = self.previous.as_mut() {
            let track = self.tracks.get_mut(&previous.track).unwrap();
            track.write(previous.into_chunk(), previous.index).await?;
        }

        // 初始化节点缓存
        self.previous = Some(Previous {
            data: self.buffer.split_to(diff_size),
            track: self.track,
            next_track: None,
            next: None,
            id: self.id,
            index
        });

        // 分片序号递加
        self.id += 1;
    }

        Ok(())
    }
}

impl Previous {
    pub fn into_chunk(&self) -> Chunk {
        Chunk {
            id: self.id,
            exist: true,
            next: Some(self.index),
            next_track: Some(self.track),
            data: self.data.freeze()
        }
    }
}
