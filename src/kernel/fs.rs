use anyhow::Result;
use std::fs::{read_dir, Metadata, ReadDir};
use std::io::SeekFrom;
use std::path::Path;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    /// let fs = Fs::new(Path::new("./a.text")).await?;
    /// ```
    pub async fn new(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .await?;
        Ok(Self(file))
    }

    /// 获取文件信息
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::Fs;
    /// use std::path::Path;
    ///
    /// let fs = Fs::new(Path::new("./a.text")).await?;
    /// let metadata = fs.stat().await?;
    /// ```
    pub async fn stat(&self) -> Result<Metadata> {
        Ok(self.0.metadata().await?)
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
    /// let mut fs = Fs::new(Path::new("./a.text")).await?;
    /// fs.write(&Bytes::from(b"hello"), 0).await?;
    /// ```
    pub async fn write(&mut self, chunk: &[u8], offset: u64) -> Result<()> {
        self.0.seek(SeekFrom::Start(offset as u64)).await?;
        self.0.write_all(chunk).await?;
        self.0.flush().await?;
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
    /// let mut fs = Fs::new(Path::new("./a.text")).await?;
    /// let size = fs.read(&mut buffer, 0).await?;
    /// ```
    pub async fn read(&mut self, chunk: &mut [u8], offset: u64) -> Result<usize> {
        self.0.seek(SeekFrom::Start(offset as u64)).await?;
        Ok(self.0.read(chunk).await?)
    }
}

/// 文件是否存在
///
/// 该函数不能用于目录，
/// 只能用于获取文件是否存在
pub async fn exists(path: &Path) -> bool {
    File::open(path).await.is_ok()
}

/// 读取目录所有条目
///
/// 返回可迭代的条目流
pub fn readdir(path: &Path) -> Result<ReadDir> {
    Ok(read_dir(path)?)
}
