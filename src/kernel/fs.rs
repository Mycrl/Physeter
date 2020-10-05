use anyhow::Result;
use std::fs::{read_dir, ReadDir};
use std::fs::{File, OpenOptions, Metadata};
use std::io::{Read, SeekFrom, Seek, Write};
use std::path::Path;

/// 文件
///
/// 文件句柄抽象
/// 内部维护写入读取缓冲区，
/// 用于优化写入读取的系统调用
pub struct Fs {
    file: File,
    cursor: u64
}

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
        Ok(Self {
            cursor: 0,
            file
        })
    }
    
    /// 获取文件信息
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::Fs;
    /// use std::path::Path;
    ///
    /// let fs = Fs::new(Path::new("./a.text"))?;
    /// let metadata = fs.stat()?;
    /// ```
    pub fn stat(&self) -> Result<Metadata> {
        Ok(self.file.metadata()?)
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
        self.file.set_len(size)?;
        self.seek(0)?;
        Ok(())
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
        self.seek(offset)?;
        self.file.write_all(chunk)?;
        self.file.flush()?;
        self.cursor_next(chunk.len());
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
        self.seek(offset)?;
        let size = self.file.read(chunk)?;
        self.cursor_next(size);
        Ok(size)
    }

    /// 从文件中读取数据到缓冲区
    ///
    /// 读取会保证读取缓冲区长度，
    /// 如果无法满足则会导致panic
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
    /// fs.promise_read(&mut buffer, 0)?;
    /// ```
    pub fn promise_read(&mut self, chunk: &mut [u8], offset: u64) -> Result<()> {
        self.seek(offset)?;
        self.file.read_exact(chunk)?;
        self.cursor_next(chunk.len());
        Ok(())
    }

    /// 设置内部游标
    ///
    /// 通过检查偏移是否为内部游标，
    /// 达到减少系统调用的目的
    #[rustfmt::skip]
    fn seek(&mut self, offset: u64) -> Result<()> {
        if offset == self.cursor { return Ok(()) }
        self.file.seek(SeekFrom::Start(offset))?;
        self.cursor = offset;
        Ok(())
    }

    /// 内部游标推进
    ///
    /// 将操作位传递给内部游标
    fn cursor_next(&mut self, size: usize) {
        self.cursor += size as u64;
    }
}

/// 读取目录所有条目
///
/// 返回可迭代的条目流
pub fn readdir(path: &Path) -> Result<ReadDir> {
    Ok(read_dir(path)?)
}