use anyhow::Result;
use std::fs::{read_dir, ReadDir};
use std::fs::{File, OpenOptions};
use std::io::{Read, SeekFrom, Seek, Write};
use std::path::Path;

/// 文件
///
/// 文件句柄抽象
pub struct Fs(File);

impl Fs {
    /// 创建文件类
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::Fs;
    /// use std::path::Path;
    ///
    /// let fs = Fs::new(Path::new("./a.text"))?;
    /// ```
    pub fn new(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        Ok(Self(file))
    }

    /// 调整文件大小
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::Fs;
    /// use std::path::Path;
    ///
    /// let fs = Fs::new(Path::new("./a.text"))?;
    /// fs.resize(0)?;
    /// ```
    pub fn resize(&mut self, size: u64) -> Result<()> {
        Ok(self.0.set_len(size)?)
    }

    /// 将缓冲区写入文件
    ///
    /// 写入操作具有原子性，
    /// 这会将缓冲区完全写入到文件中
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::Fs;
    /// use std::path::Path;
    /// use bytes::Bytes;
    ///
    /// let mut fs = Fs::new(Path::new("./a.text"))?;
    /// fs.write(&Bytes::from(b"hello"), 0)?;
    /// ```
    pub fn write(&mut self, chunk: &[u8], offset: u64) -> Result<()> {
        self.0.seek(SeekFrom::Start(offset))?;
        self.0.write_all(chunk)?;
        self.0.flush()?;
        Ok(())
    }

    /// 从文件读入数据到缓冲区
    ///
    /// 读取并非完全读取指定长度，
    /// 这里返回已经读入的长度
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::Fs;
    /// use std::path::Path;
    /// use bytes::BytesMut;
    ///
    /// let buffer = [0u8; 1024];
    /// let mut fs = Fs::new(Path::new("./a.text"))?;
    /// let size = fs.read(&mut buffer, 0)?;
    /// ```
    pub fn read(&mut self, chunk: &mut [u8], offset: u64) -> Result<usize> {
        self.0.seek(SeekFrom::Start(offset))?;
        Ok(self.0.read(chunk)?)
    }

    pub fn promise_read(&mut self, chunk: &mut [u8], offset: u64) -> Result<()> {
        self.0.seek(SeekFrom::Start(offset))?;
        self.0.read_exact(chunk)?;
        Ok(())
    }
}

/// 读取目录所有条目
///
/// 返回可迭代的条目流
pub fn readdir(path: &Path) -> Result<ReadDir> {
    Ok(read_dir(path)?)
}
