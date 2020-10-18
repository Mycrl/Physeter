//! # Kernel
//! 
//! 
//! 数据以固定大小`(4KB)`分片写入轨道文件，
//! 使用轨道文件的目的是为了兼容部分文件系统的单文件最大容量，
//! 轨道文件头部保存了当前轨道已经释放的块链表，
//! 保存尾部的目的是为了链表的快速追加，
//! 每个分片内部具有链表形式的下个分片位置以及当前分片内容长度，
//! 这虽然会导致一些空间浪费，
//! 但这是无法避免的.
//! 
//! ```
//!     
//!         |-  track header -|                /------------------------------/
//!         +-----------------+  +-----------------------------+       +----------------------+
//!         | U64 | U64 | U64 |  | 4KB | 4KB | 4KB | 4KB | 4KB >       | U16 | U64 | * (data) >
//!         +-----------------+  +-----------------------------+       +----------------------+
//!             |     |     |-> data size                                  |     |-> next chunk offset
//!             |     |-> free chunk list last offset                      |-> chunk data size (if full is 0)
//!             |-> free chunk list first offset
//! ```
//! 
//! 轨道内部并不实现文件分配表，
//! 文件分配表由外部KV存储维护，
//! 轨道文件可以存储在不同位置以至于可以存储到不同磁盘，
//! 但是不影响索引合并，
//! 这是为了现实情况需要将文件存储在不同位置而存在的特性，
//! 当文件存储在不同磁盘时，
//! 会为每个磁盘指定一个单独的线程执行读写操作，
//! 这样可以最大化利用多磁盘IO叠加.
//! 

mod chunk;
mod disk;
mod index;
mod track;
pub mod fs;

use disk::Disk;
use index::Index;
use anyhow::{anyhow, Result};
use std::io::{Read, Write};
use std::rc::Rc;

/// 核心配置
///
/// `directory` 存储目录  
/// `track_size` 轨道文件最大长度  
/// `chunk_size` 分片最大长度
pub struct KernelOptions {
    pub track_size: u64,
    pub chunk_size: u64,
    pub path: String,
}

/// 存储核心
pub struct Kernel {
    disk: Disk,
    index: Index
}

impl Kernel {
    /// 创建实例
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::Kernel;
    ///
    /// let mut kernel = Kernel::new(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ).unwrap();
    /// ```
    pub fn new(path: String, track_size: u64) -> Result<Self> {
        let configure = Rc::new(KernelOptions::from(path, track_size));
        let mut disk = Disk::new(configure.clone());
        disk.init()?;
        Ok(Self {
            index: Index::new(&configure)?,
            disk,
        })
    }

    /// 读取数据
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::Kernel;
    ///
    /// let mut kernel = Kernel::new(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ).unwrap();
    ///
    /// let file = std::fs::File::open("test.mp4")?;
    /// kernel.read(b"test", file).unwrap();
    /// ```
    pub fn read(&mut self, key: &[u8], stream: impl Write) -> Result<()> {
        match self.index.get(key)? {
            Some(x) => self.disk.read(stream, x),
            _ => Err(anyhow!("not found")),
        }
    }

    /// 写入数据
    ///
    /// # Examples
    ///
    // ```no_run
    /// use super::Kernel;
    ///
    /// let mut kernel = Kernel::new(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ).unwrap();
    ///
    /// let file = std::fs::File::open("test.mp4")?;
    /// kernel.write(b"test", file).unwrap();
    /// ```
    #[rustfmt::skip]
    pub fn write(&mut self, key: &[u8], stream: impl Read) -> Result<()> {
        if self.index.has(key)? { return Err(anyhow!("not empty")); }
        self.index.set(key, &self.disk.write(stream)?)
    }

    /// 删除数据
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::Kernel;
    ///
    /// let mut kernel = Kernel::new(
    ///     Path::new("./.static"), 
    ///     1024 * 1024 * 1024 * 1
    /// ).unwrap();
    ///
    /// kernel.delete(b"test").unwrap();
    /// ```
    pub fn delete(&mut self, key: &[u8]) -> Result<()> {
        match self.index.get(key)? {
            None => Err(anyhow!("not found")),
            Some(x) => {
                self.disk.remove(&x)?;
                self.index.remove(key)
            }
        }
    }
}

impl KernelOptions {
    pub fn from(path: String, track_size: u64) -> Self {
        Self {
            chunk_size: 4096,
            track_size,
            path,
        }
    }
}
