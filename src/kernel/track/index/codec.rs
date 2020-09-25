//!
//! size | name_size | name | ...index |
//! u32  | u8        | *    | u64      |

use bytes::{BytesMut, Buf, BufMut};

/// 索引项
///
/// `name` 索引键  
/// `index` 分片索引列表
pub struct Value<'a> {
    name: &'a str,
    index: Vec<u64>
}

/// 编解码器
/// 
/// 索引缓冲区的编解码
///
/// `buffer` 内部缓冲区  
pub struct Codec {
    buffer: BytesMut
}

impl Codec {
    /// 创建实例
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Codec};
    ///
    /// let codec = Codec::new();
    /// ```
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new(),
        }
    }

    /// 解码索引
    ///
    /// 传递缓冲区分片，
    /// 尝试解码为索引数据
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Codec};
    ///
    /// let mut codec = Codec::new();
    /// let list = codec.decoder(&b"hello");
    /// ```
    pub fn decoder(&mut self, chunk: &[u8], offset: u64) -> Vec<(u32, u64, String)> {
        self.buffer.extend_from_slice(chunk);
        let values = Vec::new();

        // 无限循环
        // 直到缓冲区无法解码
    loop {
        let buffer_size = self.buffer.len();

        // 缓冲区长度不足
        // 直接跳出循环
        if buffer_size < 6 {
            break;
        }

        // 获取索引项总长度
        let size = u32::from_be_bytes([
            self.buffer[0],
            self.buffer[1],
            self.buffer[2],
            self.buffer[3]
        ]);

        // 检查缓冲区是否足够解码
        // 如果不足索引项长度则跳出
        if buffer_size < size as usize {
            break;
        }

        // 缓冲区游标前进u32
        self.buffer.advance(4);

        // 获取索引键长度
        // 解码出索引键
        let name_size = self.buffer.get_u8() as usize;
        let name_buf = self.buffer[0..name_size].to_vec();
        let name = unsafe { String::from_utf8_unchecked(name_buf) };
        self.buffer.advance(name_size);

        // 计算索引列表长度
        // 计算索引列表头部位置
        // 缓冲区游标消耗掉索引列表
        let head_size = 5 + name_size;
        let index = offset + (head_size as u64);
        let index_size = size - (head_size as u32);
        self.buffer.advance(index_size as usize);

        values.push((
            index_size,
            index,
            name
        ))
    }

        values
    }

    /// 编码索引
    ///
    /// 传递整个索引列表
    /// 编码为索引项缓冲区
    /// TODO: 有优化空间，如果索引列表很长，
    /// 会占用很多内存空间
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Codec, Value};
    ///
    /// let mut codec = Codec::new();
    /// let buf = codec.encoder(Value {
    ///     name: &"hello",
    ///     index: vec![0, 4]
    /// });
    /// ```
    pub fn encoder(&self, value: Value) -> &[u8] {
        let packet = BytesMut::new();

        // 计算索引项总长度
        let name_size = value.name.len();
        let size = value.index.len() * 8 + name_size + 5;

        // 写入长度
        // 写入索引键长度
        // 写入索引键
        packet.put_u32(size as u32);
        packet.put_u8(name_size as u8);
        packet.put(value.name.as_bytes());

        // 写入索引列表
        for index in value.index {
            packet.put_u64(index);
        }

        &packet
    }
}
