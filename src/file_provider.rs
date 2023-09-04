use std::io::{Read, Seek, SeekFrom, Write};
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use mockall::automock;

#[automock]
#[async_trait]
pub trait FileProvider: Send {
    async fn read(&mut self, offset: usize, length: usize) -> Vec<u8>;
    async fn write(&mut self, offset: usize, data: Vec<u8>);
}

pub struct TokioFileProvider {
    file: tokio::fs::File,
}

impl TokioFileProvider {
    pub fn new(file: tokio::fs::File) -> Self {
        return TokioFileProvider { file };
    }
}

#[async_trait]
impl FileProvider for TokioFileProvider {
    async fn read(&mut self, offset: usize, length: usize) -> Vec<u8> {
        let mut piece_buff = vec![0u8; length];
        self.file.seek(SeekFrom::Start(offset as u64)).await.unwrap();
        self.file.read_exact(&mut piece_buff).await.unwrap();
        return piece_buff;
    }

    async fn write(&mut self, offset: usize, data: Vec<u8>) {
        self.file.seek(SeekFrom::Start(offset as u64)).await.unwrap();
        self.file.write_all(&*data).await.unwrap();
    }
}

pub struct StdFileProvider {
    file: std::fs::File,
}

impl StdFileProvider {
    pub fn new(file: std::fs::File) -> Self {
        return StdFileProvider { file };
    }
}

#[async_trait]
impl FileProvider for StdFileProvider {
    async fn read(&mut self, offset: usize, length: usize) -> Vec<u8> {
        let mut piece_buff = vec![0u8; length];
        self.file.seek(SeekFrom::Start(offset as u64)).unwrap();
        self.file.read_exact(&mut piece_buff).unwrap();
        return piece_buff;
    }

    async fn write(&mut self, offset: usize, data: Vec<u8>) {
        self.file.seek(SeekFrom::Start(offset as u64)).unwrap();
        self.file.write_all(&*data).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempfile;
    use super::*;

    #[tokio::test]
    async fn test_read_and_write() {
        let temp_file = tempfile().unwrap();
        let mut file_provider = TokioFileProvider {
            file: tokio::fs::File::from_std(temp_file),
        };

        let block_data = vec![1, 2, 3, 4];
        file_provider.write(0, block_data.clone()).await;
        let read_data = file_provider.read(0, block_data.len()).await;
        assert_eq!(block_data, read_data);
    }
}



