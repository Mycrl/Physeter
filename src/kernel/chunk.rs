use super::KernelOptions;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::rc::Rc;

/// 分片
///
/// 分片以链表形式表示连续存储
/// 
/// `next` 下个分片索引  
/// `data` 分片数据  
#[derive(Clone, Debug)]
pub struct Chunk {
    pub next: Option<u64>,
    pub data: Bytes,
}

/// 分片编解码器
///
/// 将分片编码为缓冲区
/// 或者将缓冲区解码为分片.
///
/// #### diff_size
/// 分片内部最大数据长度，分片固定头长度为17，
/// 所以这里使用分片长度减去17.
pub struct Codec {
    chunk_size: usize,
    diff_size: u64,
}

impl Codec {
    /// 创建编解码器
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Codec, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// Codec::new(&options);
    /// ````
    pub fn new(options: Rc<KernelOptions>) -> Self {
        Self {
            diff_size: options.chunk_size - 10,
            chunk_size: options.chunk_size as usize
        }
    }

    /// 编码分片
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Chunk, Codec, KernelOptions};
    /// use bytes::Bytes;
    ///
    /// let chunk = Chunk {
    ///     next: Some(17),
    ///     data: Bytes::from_static(b"hello"),
    /// };
    ///
    /// let options = KernelOptions::default();
    /// let codec = Codec::new(&options);
    /// let packet = codec.encoder(chunk.clone());
    /// ```
    #[rustfmt::skip]
    pub fn encoder(&self, chunk: &Chunk) -> Bytes {
        let mut packet = BytesMut::new();

        let size = match chunk.data.len() == self.diff_size as usize {
            false => chunk.data.len() as u16,
            true => 0,
        };

        let next = match chunk.next {
            Some(next) => next,
            None => 0,
        };

        packet.put_u64(next);
        packet.put_u16(size);
        packet.extend_from_slice(&chunk.data);

        if packet.len() < self.chunk_size {
            packet.resize(self.chunk_size, 0);
        }

        packet.freeze()
    }

    /// 解码分片
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Chunk, Codec, KernelOptions};
    /// use bytes::Bytes;
    ///
    /// let chunk = Chunk {
    ///     next: Some(17),
    ///     data: Bytes::from_static(b"hello"),
    /// };
    ///
    /// let options = KernelOptions::default();
    /// let codec = Codec::new(&options);
    /// let packet = codec.encoder(chunk.clone());
    /// let result = codec.decoder(packet.clone());
    ///
    /// assert_eq!(result.id, chunk.id);
    /// assert_eq!(result.exist, chunk.exist);
    /// assert_eq!(result.next, chunk.next);
    /// assert_eq!(result.next_track, chunk.next_track);
    /// assert_eq!(result.data, chunk.data);
    /// ```
    #[rustfmt::skip]
    pub fn decoder(&self, mut chunk: Bytes) -> Chunk {
        let source_next = chunk.get_u64();
        let source_size = chunk.get_u16();

        let size = match source_size {
            0 => self.diff_size as usize,
            _ => source_size as usize,
        };

        let data = chunk.slice(0..size);

        let next = match source_next == 0 {
            false => Some(source_next),
            true => None,
        };

        Chunk {
            next,
            data,
        }
    }
}
