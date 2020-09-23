mod reader;
mod writer;

pub use super::{fs::readdir, chunk::Chunk};
pub use super::{track::Track, KernelOptions};
use std::collections::HashMap;
use anyhow::Result;

/// 轨道列表
pub type Tracks<'a> = HashMap<u16, Track<'a>>;

/// 内部存储
///
/// 管理所有轨道的读取和写入
///
/// `options` 配置  
/// `tracks` 轨道列表
pub struct Disk<'a> {
    options: &'a KernelOptions<'a>,
    tracks: Tracks<'a>,
}

impl<'a> Disk<'a> {
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
    pub fn new(options: &'a KernelOptions<'_>) -> Self {
        Self {
            tracks: HashMap::new(),
            options,
        }
    }

    /// 初始化
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Disk, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut disk = Disk::new(options);
    /// disk.init().await?;
    /// ```
    pub async fn init(&mut self) -> Result<()> {
        let dir_iter = readdir(self.options.directory)?;
        let mut track_count: i32 = 0;

        // 读取目录的所有轨道文件，
        // 将找到的轨道索引创建为轨道类，
        // 并推入内部轨道列表
        for dir in readdir(self.options.directory)? {
            if let Ok(name) = dir?.file_name().into_string() {
                if name.ends_with(".track") {
                    if let Ok(track_id) = name.replace(".track", "").parse::<u16>() {
                        self.create_track(track_id).await?;
                        track_count += 1;
                    }
                }
            }
        }

        // 如果未找到轨道
        // 则创建初始轨道
        if track_count == 0 {
            self.create_track(0).await?;
        }

        Ok(())
    }

    /// 创建轨道
    ///
    /// 创建轨道类并初始化，
    /// 将轨道添加到内部的轨道列表
    async fn create_track(&mut self, id: u16) -> Result<()> {
        let mut track = Track::new(id, self.options).await?;
        track.init().await?;
        self.tracks.insert(id, track);
        Ok(())
    }
}
