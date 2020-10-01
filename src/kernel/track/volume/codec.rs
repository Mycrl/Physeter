use super::KernelOptions;
use bytes::{BufMut, BytesMut, Buf};
use std::rc::Rc;

pub struct Header {
    pub index_first: u64,
    pub index_end: u64,
    pub release_first: u64,
    pub release_end: u64
}

/// 分片
///
/// 分片以链表形式表示连续存储
///
/// `id` 分片ID  
/// `exist` 是否有效  
/// `next` 下个分片索引  
/// `next_track` 下个分片轨道  
/// `data` 分片数据  
#[derive(Clone, Debug)]
pub struct Chunk {
    pub next: Option<u64>,
    pub data: BytesMut,
}

/// 分片编解码器
///
/// 将分片编码为缓冲区
/// 或者将缓冲区解码为分片.
///
/// #### diff_size
/// 分片内部最大数据长度，分片固定头长度为8，
/// 所以这里使用分片长度减去8.
pub struct Codec {
    chunk_size: usize,
    diff_size: u16,
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
    ///     id: 0,
    ///     exist: true,
    ///     next: Some(17),
    ///     next_track: None,
    ///     data: Bytes::from_static(b"hello"),
    /// };
    ///
    /// let options = KernelOptions::default();
    /// let codec = Codec::new(&options);
    /// let packet = codec.encoder(chunk.clone());
    /// ```
    pub fn encoder(&self, chunk: Chunk) -> BytesMut {
        let mut packet = BytesMut::new();

        let size = match chunk.data.len() == self.diff_size as usize {
            false => chunk.data.len() as u16,
            true => 0,
        };

        let next = match chunk.next {
            Some(next) => next,
            None => 0,
        };

        packet.put_u16(size);
        packet.put_u64(next);
        packet.put(chunk.data);

        if packet.len() < self.chunk_size {
            packet.resize(self.chunk_size, 0);
        }

        packet
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
    ///     id: 0,
    ///     exist: true,
    ///     next: Some(17),
    ///     next_track: None,
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
    pub fn decoder(&self, chunk: &Vec<u8>) -> Chunk {
        let source_size = u16::from_be_bytes([
            chunk[0],
            chunk[1]
        ]);

        let source_next = u64::from_be_bytes([
            chunk[2],
            chunk[3],
            chunk[4],
            chunk[5],
            chunk[6],
            chunk[7],
            chunk[8],
            chunk[9]
        ]);

        let size = match source_size {
            0 => self.diff_size,
            _ => source_size,
        } as usize;

        let next = match source_next == 0 {
            false => Some(source_next),
            true => None,
        };

        let data = BytesMut::from(&chunk[9..size]);

        Chunk {
            next,
            data
        }
    }

    pub fn decoder_header(&self, mut chunk: &[u8]) -> Header {
        let index_first = chunk.get_u64();
        let index_end = chunk.get_u64();
        let release_first = chunk.get_u64();
        let release_end = chunk.get_u64();

        Header {
            index_first, 
            index_end,
            release_first,
            release_end
        }
    }
}
