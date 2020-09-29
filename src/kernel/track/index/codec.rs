//!
//! size | name_size | name | ...index |
//! u32  | u8        | *    | u64      |

use bytes::{BytesMut, BufMut};
use super::KernelOptions;

pub struct Value {
    index_size: u32,
    index: u64,
}

/// 编解码器
/// 
/// 索引缓冲区的编解码
pub struct Codec {
    data_size: u16
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
    pub fn new(options: &KernelOptions) -> Self {
        Self {
            data_size: options.chunk_size - 10,
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
    pub fn decoder(&mut self, chunk: BytesMut, offset: u64) -> (String, usize, Value) {

        // 获取索引项总长度
        let size = u32::from_be_bytes([
            chunk[0],
            chunk[1],
            chunk[2],
            chunk[3]
        ]) as usize;

        // 索引项的长度总是幂等
        // 每个索引所占空间为分片长度的倍数
        let size_diff = size % self.data_size as usize;

        // 获取索引键长度
        // 解码出索引键
        let name_end = chunk[4] as usize + 5;
        let name_buf = chunk[5..name_end].to_vec();
        let name = unsafe { String::from_utf8_unchecked(name_buf) };

        // 计算索引列表长度
        // 计算索引列表头部位置
        // 缓冲区游标消耗掉索引列表
        let head_size = 5 + chunk[4] as usize;
        let index = offset + (head_size as u64);
        let index_size = (size - head_size) as u32;

        (name, size + size_diff, Value {
            index_size,
            index
        })
    }

    /// 编码索引
    ///
    /// 传递整个索引列表
    /// 编码为索引项缓冲区
    ///
    /// TODO: 
    /// 有优化空间，如果索引列表很长，
    /// 会占用很多内存空间
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Codec, Value};
    ///
    /// let mut codec = Codec::new();
    /// let buf = codec.encoder(&"hello", vec![0, 10]);
    /// ```
    pub fn encoder(&self, key: &str, index_list: Vec<u64>) -> BytesMut {
        let mut packet = BytesMut::new();

        // 计算索引项总长度
        let name_size = key.len();
        let size = index_list.len() * 8 + name_size + 5;

        // 写入长度
        // 写入索引键长度
        // 写入索引键
        packet.put_u32(size as u32);
        packet.put_u8(name_size as u8);
        packet.put(key.as_bytes());

        // 写入索引列表
        for index in index_list {
            packet.put_u64(index);
        }

        packet
    }
}
