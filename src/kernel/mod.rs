mod chunk;
mod disk;
pub mod fs;
mod index;
mod track;

use anyhow::{anyhow, Result};
use disk::Disk;
use index::Index;
use std::io::{Read, Write};
use std::{path::Path, rc::Rc};

/// 核心配置
///
/// `directory` 存储目录  
/// `track_size` 轨道文件最大长度  
/// `chunk_size` 分片最大长度
pub struct KernelOptions {
    pub path: &'static Path,
    pub track_size: u64,
    pub chunk_size: u64,
}

/// 存储核心
pub struct Kernel {
    disk: Disk,
    index: Index,
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
    pub fn new(path: &'static Path, track_size: u64) -> Result<Self> {
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
            Some(x) => self.disk.read(stream, &x),
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
    pub fn from(path: &'static Path, track_size: u64) -> Self {
        Self {
            chunk_size: 1024 * 4,
            track_size,
            path,
        }
    }
}
