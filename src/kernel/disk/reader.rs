use super::Tracks;
use anyhow::Result;
use bytes::Bytes;

/// 读取流
///
/// 从轨道中读取数据，
/// 游标由内部维护
///
/// `tracks` 轨道列表  
/// `track` 轨道索引  
/// `index` 节点索引
pub struct Reader {
    tracks: Tracks,
    track: u16,
    index: u64,
}

impl Reader {
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
    pub fn new(track: u16, index: u64, tracks: Tracks) -> Self {
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
    /// let data = reader.read()?;
    /// ```
    #[rustfmt::skip]
    pub fn read(&mut self) -> Result<(Bytes, bool)> {
        let mut tracks = self.tracks.borrow_mut();
        let track = tracks.get_mut(&self.track).unwrap();
        let chunk = track.read(self.index)?;

        // 如果链表还未结束
        // 将下个位置保存到内部游标
        if let (Some(next), Some(track_id)) = (chunk.next, chunk.next_track) {
            self.track = track_id;
            self.index = next;
        }

        Ok((
            chunk.data, 
            chunk.next.is_some()
        ))
    }
}
