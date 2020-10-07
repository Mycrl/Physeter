mod chunk;
mod disk;
mod index;
mod track;
pub mod fs;

use index::Index;
use std::{path::Path, rc::Rc};
use std::io::{Read, Write};
use anyhow::{Result, anyhow};
use disk::Disk;

/// 核心配置
///
/// `directory` 存储目录  
/// `track_size` 轨道文件最大长度  
/// `chunk_size` 分片最大长度  
/// `max_memory` 最大内存使用量
pub struct KernelOptions {
    pub directory: &'static Path,
    pub track_size: u64,
    pub chunk_size: u64,
    pub max_memory: u64,
}

/// 存储核心
///
/// `index` 索引类  
/// `disk` 磁盘类
pub struct Kernel {
    index: Index,
    disk: Disk
}

impl Kernel {
    /// 创建实例
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Kernel, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut kernel = Kernel::new(&options)?;
    /// ```
    pub fn new(options: KernelOptions) -> Result<Self> {
        let configure = Rc::new(options);
        Ok(Self {
            index: Index::new(&configure)?,
            disk: Disk::new(configure.clone())
        })
    }

    /// 打开实例
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Kernel, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut kernel = Kernel::new(&options)?;
    ///
    /// kernel.open()?;
    /// ```
    pub fn open(&mut self) -> Result<()> {
        self.disk.init()?;
        Ok(())
    }

    /// 读取数据
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Kernel, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut kernel = Kernel::new(&options)?;
    ///
    /// kernel.open()?;
    ///
    /// let file = std::fs::File::open("test.mp4")?;
    /// kernel.read("test", file)?;
    /// ```
    pub fn read(&mut self, key: &[u8], stream: impl Write) -> Result<()> {
        match self.index.get(key)? {
            Some(x) => self.disk.read(stream, &x),
            _ => Err(anyhow!("not found"))
        }
    }

    /// 写入数据
    ///
    /// # Examples
    ///
    // ```no_run
    /// use super::{Kernel, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut kernel = Kernel::new(&options)?;
    ///
    /// kernel.open()?;
    ///
    /// let file = std::fs::File::open("test.mp4")?;
    /// kernel.write("test", file)?;
    /// ```
    pub fn write(&mut self, key: &[u8], stream: impl Read) -> Result<()> {
        if self.index.has(key)? { return Err(anyhow!("not empty")) }
        self.index.set(key, &self.disk.write(stream)?);
        Ok(())
    }

    /// 删除数据
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::{Kernel, KernelOptions};
    ///
    /// let options = KernelOptions::default();
    /// let mut kernel = Kernel::new(&options)?;
    ///
    /// kernel.open()?;
    ///
    /// kernel.delete("test")?;
    /// ```
    pub fn delete(&mut self, key: &[u8]) -> Result<()> {
        match self.index.get(key)? {
            None => Err(anyhow!("not found")),
            Some(x) => {
                self.disk.remove(&x)?;
                self.index.remove(key);
                Ok(())
            }
        }
    }
}

impl<'a> Default for KernelOptions {
    fn default() -> Self {
        Self {
            track_size: 1024 * 1024 * 1024 * 50,
            max_memory: 1024 * 1024 * 1024,
            directory: Path::new("./"),
            chunk_size: 1024 * 4,
        }
    }
}
