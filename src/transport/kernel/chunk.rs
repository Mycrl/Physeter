use super::KernelOptions;
use std::rc::Rc;
use bytes::{
    BufMut, 
    Bytes, 
    BytesMut
};

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
    diff_size: usize,
}

impl Codec {
    /// 创建编解码器
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Codec, KernelOptions};
    /// use std::rc::Rc;
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// Codec::new(Rc::new(options));
    /// ````
    pub fn new(options: Rc<KernelOptions>) -> Self {
        Self {
            diff_size: (options.chunk_size - 10) as usize,
            chunk_size: options.chunk_size as usize,
        }
    }

    /// 编码分片
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Chunk, Codec, KernelOptions};
    /// use std::rc::Rc;
    /// use bytes::Bytes;
    ///
    /// let chunk = Chunk {
    ///     next: Some(17),
    ///     data: Bytes::from_static(b"hello"),
    /// };
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// let codec = Codec::new(options);
    /// let packet = codec.encoder(chunk.clone());
    /// ```
    #[rustfmt::skip]
    pub fn encoder(&self, next_offset: Option<u64>, chunk: &[u8]) -> Bytes {
        let mut packet = BytesMut::new();

        let size = match chunk.len() == self.diff_size {
            false => chunk.len() as u16,
            true => 0,
        };

        let next = match next_offset {
            Some(next) => next,
            None => 0,
        };

        packet.put_u64(next);
        packet.put_u16(size);
        packet.extend_from_slice(&chunk);

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
    /// use std::rc::Rc;
    /// use bytes::Bytes;
    ///
    /// let chunk = Chunk {
    ///     next: Some(17),
    ///     data: Bytes::from_static(b"hello"),
    /// };
    ///
    /// let options = Rc::new(KernelOptions::from(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ));
    ///
    /// let codec = Codec::new(options);
    /// let packet = codec.encoder(chunk.clone());
    /// let result = codec.decoder(packet.clone());
    ///
    /// assert_eq!(result.next, chunk.next);
    /// assert_eq!(result.data, chunk.data);
    /// ```
    #[rustfmt::skip]
    pub fn decoder<'a>(&self, chunk: &'a [u8]) -> (Option<u64>, &'a [u8]) {
        assert!(chunk.len() > 10);
        let source_next = u64::from_be_bytes([
            chunk[0],
            chunk[1],
            chunk[2],
            chunk[3],
            chunk[4],
            chunk[5],
            chunk[6],
            chunk[7],
        ]);

        let source_size = u16::from_be_bytes([
            chunk[8],
            chunk[9]
        ]) as usize;

        let end_offset = match source_size {
            0 => self.diff_size,
            _ => source_size,
        } + 10;

        assert!(end_offset <= chunk.len());
        let data = &chunk[10..end_offset];

        let next = match source_next == 0 {
            false => Some(source_next),
            true => None,
        };

        (
            next,
            data
        )
    }
}
