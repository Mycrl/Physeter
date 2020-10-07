use super::{Tracks, AllocMap, KernelOptions};
use anyhow::Result;
use bytes::Bytes;
use std::rc::Rc;

/// 读取流
///
/// 从轨道中读取数据，
/// 游标由内部维护
///
/// `tracks` 轨道列表  
/// `track` 轨道索引  
/// `index` 节点索引
pub struct Reader<'a> {
    alloc_map: &'a AllocMap,
    tracks: Tracks,
    track: usize,
    index: usize,
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
    pub fn new(tracks: Tracks, alloc_map: &'a AllocMap) -> Self {
        Self {
            track: 0,
            index: 0,
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
    /// let mut tracks = HashMap::new();
    /// let mut reader = Reader::new(0, 16, &mut tracks);
    /// let data = reader.read()?;
    /// ```
    #[rustfmt::skip]
    pub fn read(&mut self) -> Result<Option<Bytes>> {
        if let Some((track_id, list)) = self.alloc_map.get(self.track) {
            if let Some(index) = list.get(self.index) {
                let mut tracks = self.tracks.borrow_mut();
                let track = tracks.get_mut(&track_id).unwrap();
                let chunk = track.read(*index)?;
                if self.index + 1 >= list.len() {
                    self.track += 1;
                    self.index = 0;
                } else {
                    self.index += 1;
                }

                Ok(Some(chunk.data))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}
