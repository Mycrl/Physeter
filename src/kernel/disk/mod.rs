mod reader;
mod writer;

pub use super::index::AllocMap;
pub use super::{fs::readdir, chunk::Chunk};
pub use super::{track::Track, KernelOptions};
use std::{cell::RefCell, rc::Rc};
use std::collections::HashMap;
use writer::{Writer, Callback};
use std::io::{Read, Write};
use reader::Reader;
use anyhow::Result;

/// 轨道列表
pub type Tracks = Rc<RefCell<HashMap<u16, Track>>>;

/// 内部存储
///
/// 管理所有轨道的读取和写入
///
/// `options` 配置  
/// `tracks` 轨道列表
pub struct Disk {
    options: Rc<KernelOptions>,
    tracks: Tracks,
}

impl Disk {
    /// 创建内部存储
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Disk, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let disk = Disk::new(options);
    /// ```
    pub fn new(options: Rc<KernelOptions>) -> Self {
        Self {
            tracks: Rc::new(RefCell::new(HashMap::new())),
            options,
        }
    }

    /// 初始化
    ///
    /// 必须对该实例调用初始化，
    /// 才能进行其他操作
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Disk, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut disk = Disk::new(options);
    /// disk.init()?;
    /// ```
    #[rustfmt::skip]
    pub fn init(&mut self) -> Result<()> {
        let mut track_count: i32 = 0;

        // 读取目录的所有轨道文件，
        // 将找到的轨道索引创建为轨道类，
        // 并推入内部轨道列表
        for dir in readdir(self.options.directory)? {
            if let Ok(name) = dir?.file_name().into_string() {
                if name.ends_with(".track") {
                    if let Ok(track_id) = name.replace(".track", "").parse::<u16>() {
                        self.create_track(track_id)?;
                        track_count += 1;
                    }
                }
            }
        }

        // 如果未找到轨道
        // 则创建初始轨道
        if track_count == 0 {
            self.create_track(1)?;
        }

        Ok(())
    }

    /// 打开读取流
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Disk, KernelOptions};
    /// use std::fs::File;
    ///
    /// let options = KernelOptions::default();
    /// let mut disk = Disk::new(options);
    /// disk.init()?;
    ///
    /// let file = File::open("test.mp4");
    /// disk.read(file, 0, 19)?;
    /// ```
    #[rustfmt::skip]
    pub fn read(&mut self, mut stream: impl Write, alloc_map: &AllocMap) -> Result<()> {
        let mut reader = Reader::new(self.tracks.clone(), alloc_map);

        // 无限循环
        // 将轨道数据全部读取
        // 写入外部流中
    loop {
        match reader.read()? {
            Some(data) => stream.write_all(&data)?,
            None => break
        }
    }

        // 写入完成之后
        // 清空尾部缓冲区，
        // 将所有数据推入目的地
        stream.flush()?;
        Ok(())
        
    }

    /// 打开写入流
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Disk, KernelOptions};
    /// use std::fs::File;
    ///
    /// let options = KernelOptions::default();
    /// let mut disk = Disk::new(options);
    /// disk.init()?;
    ///
    /// let file = File::open("test.mp4");
    /// let (track, index) = disk.write(file)?;
    /// ```
    #[rustfmt::skip]
    pub fn write<'a>(&mut self, mut stream: impl Read) -> Result<AllocMap> {
        let mut writer = Writer::new(self.tracks.clone(), self.options.clone());
        let mut buffer = [0; 2048];
        let mut size = 1;

        // 无限循环
        // 读取外部源写入轨道
    loop {
        
        // 读取外部流数据
        // 检查上次读取长度是否为空
        // 如果不为空则不做重复调用
        if size != 0 {
            size = stream.read(&mut buffer)?;   
        }
        
        // 检查数据为空的情况
        let data = if size > 0 {
            Some(&buffer[0..size]) 
        } else { 
            None
        };
        
        // 向轨道写入数据
        // 处理写入返回，如创建新轨道，
        // 如果轨道返回头部索引，说明写入完成
        if let Some(callback) = writer.write(data)? {
            match callback {
                Callback::CreateTrack(track) => self.create_track(track)?,
                Callback::Done => return Ok(writer.alloc_map),
                _ => ()
            }
        }
    }
    }

    /// 删除数据
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Disk, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut disk = Disk::new(options);
    /// disk.init()?;
    ///
    /// disk.remove(0, 16)?;
    /// ```
    #[rustfmt::skip]
    pub fn remove(&mut self, alloc_map: &AllocMap) -> Result<()> {
        let mut tracks = self.tracks.borrow_mut();
        for (track_id, list) in alloc_map {
            if let Some(track) = tracks.get_mut(track_id) {
                track.remove(list)?;
            }
        }

        Ok(())
    }

    /// 创建轨道
    ///
    /// 创建轨道类并初始化，
    /// 将轨道添加到内部的轨道列表
    #[rustfmt::skip]
    fn create_track(&mut self, id: u16) -> Result<()> {
        let mut track = Track::new(id, self.options.clone())?;
        track.init()?;
        self.tracks
            .borrow_mut()
            .insert(id, track);
        Ok(())
    }
}
