use super::Tracks;
use anyhow::Result;
use bytes::Bytes;
use std::collections::HashMap;

/// 读取流
///
/// 从轨道中读取数据，
/// 游标由内部维护
///
/// `tracks` 轨道列表  
/// `track` 轨道索引  
/// `index` 节点索引
pub struct Reader<'a> {
    tracks: &'a mut Tracks<'a>,
    track: u16,
    index: u64,
}

impl<'a> Reader<'a> {
    /// 创建读取流
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::Reader;
    /// use std::collections::HashMap;
    ///
    /// let mut tracks = HashMap::new();
    /// let reader = Reader::new(0, 16, &mut tracks);
    /// ```
    pub fn new(track: u16, index: u64, tracks: &'a mut Tracks<'a>) -> Self {
        Self {
            tracks,
            track,
            index,
        }
    }

    /// 读取数据
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::Reader;
    /// use std::collections::HashMap;
    ///
    /// let mut tracks = HashMap::new();
    /// let mut reader = Reader::new(0, 16, &mut tracks);
    /// let data = reader.read().await?;
    /// ```
    pub async fn read(&mut self) -> Result<Option<Bytes>> {
        let track = self.tracks.get_mut(&self.track).unwrap();
        let chunk = track.read(self.index).await?;

        // 如果链表还未结束
        // 将下个节点位置保存到内部游标
        if let Some(next) = chunk.next {
            self.index = next
        }

        // 如果链表还未结束
        // 将下个轨道位置保存到内部游标
        if let Some(track_id) = chunk.next_track {
            self.track = track_id
        }

        Ok(match chunk.next {
            Some(_) => Some(chunk.data),
            None => None,
        })
    }
}
