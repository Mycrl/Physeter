use super::{AllocMap, Tracks};
use anyhow::Result;

/// 读取流
///
/// 从轨道中读取数据，
/// 游标由内部维护
pub struct Reader {
    alloc_map: AllocMap,
    track_index: usize,
    alloc_size: usize,
    track_id: usize,
    tracks: Tracks,
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
    /// let reader = Reader::new(HashMap::new(), HashMap::new());
    /// ```
    pub fn new(tracks: Tracks, alloc_map: AllocMap) -> Self {
        Self {
            alloc_size: alloc_map.len(),
            track_index: 0,
            track_id: 0,
            alloc_map,
            tracks,
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
    /// let reader = Reader::new(HashMap::new(), HashMap::new());
    /// let data = reader.read().unwrap();
    /// ```
    #[rustfmt::skip]
    pub fn read(&mut self) -> Result<Option<Vec<u8>>> {
        
        // 如果轨道遍历完成
        // 则返回`None`表示读取为空
        if self.track_id >= self.alloc_size {
            return Ok(None);
        }

        // 获取轨道分配表索引
        // 获取分片数据内容
        let (track_id, list) = self.alloc_map.get(self.track_id).unwrap();
        let mut tracks = self.tracks.borrow_mut();
        let track = tracks.get_mut(&track_id).unwrap();
        let index = list.get(self.track_index).unwrap();
        let (next, chunk) = track.read(*index)?;
        
        // 如果没有后续分片
        // 则返回`None`表示读取为空
        if let None = next {
            return Ok(None);
        }

        // 检查是否抵达轨道尾部
        // 如果抵达尾部则前进到下个轨道
        self.track_index += 1;
        if self.track_index >= list.len() {
            self.track_index = 0;
            self.track_id += 1;
        }

        Ok(Some(
            chunk.to_vec()
        ))
    }
}
