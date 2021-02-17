use anyhow::Error;
use rill_protocol::provider::{Path, RillEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Cursor, SeekFrom, Write};
use std::mem;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt as AsyncWrite};

// [declaration: <len=u64><meta><next=u64>]
// [data: <len=u64><first_record_of_declared><next=u64>] - first == BeginStream

#[derive(Debug, PartialEq, Eq, Hash)]
enum Pointer {
    Declaration,
    Event { path: Path },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Record {
    Declaration { path: Path },
    Event { event: RillEvent },
}

/*
impl Record {
    fn pointer(&self) -> Pointer {
        match self {
            Self::Declaration { .. } => Pointer::Declaration,
            Self::Event { .. } => Pointer::Event,
        }
    }
}
*/

const ZERO: [u8; 8] = 0u64.to_be_bytes();
const ZERO_LEN: usize = mem::size_of::<u64>();

pub struct LogFile {
    buffer: Vec<u8>,
    file: File,
    last_pointer: HashMap<Pointer, u64>,
    // TODO: Use `Seek` end instead
    //cursor: usize,
}

impl LogFile {
    pub async fn open(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        // TODO: Reopen if exists? Rotate?
        let file = OpenOptions::new().read(true).write(true).open(path).await?;
        Ok(Self {
            buffer: Vec::new(),
            file,
            last_pointer: HashMap::new(),
        })
    }

    async fn go_to(&mut self, pos: u64) -> Result<(), Error> {
        self.file.seek(SeekFrom::Start(pos)).await?;
        Ok(())
    }

    async fn go_to_start(&mut self) -> Result<(), Error> {
        self.file.seek(SeekFrom::Start(0)).await?;
        Ok(())
    }

    async fn go_to_end(&mut self) -> Result<(), Error> {
        self.file.seek(SeekFrom::End(0)).await?;
        Ok(())
    }

    async fn get_position(&mut self) -> Result<u64, Error> {
        self.file
            .seek(SeekFrom::Current(0))
            .await
            .map_err(Error::from)
    }

    async fn write_record(&mut self, pointer: Pointer, record: &Record) -> Result<(), Error> {
        let new_record_position = self.get_position().await?;
        let doc = bson::to_document(record)?;
        self.buffer.clear();
        // Reserves a space for the length of data
        Write::write_all(&mut self.buffer, &ZERO)?;
        // Writes the data
        doc.to_writer(&mut self.buffer)?;
        // Updates the size of data
        let size = self.buffer.len() - ZERO_LEN;
        (&mut self.buffer[0..ZERO_LEN]).copy_from_slice(&size.to_be_bytes());
        // Reserve the space for a pointer to the next record
        Write::write_all(&mut self.buffer, &ZERO)?;
        //log::trace!("Writing buffer: {:?}", self.buffer);
        AsyncWrite::write_all(&mut self.file, &self.buffer).await?;
        // Chaining records
        let new_last_pointer = self.get_position().await? - ZERO_LEN as u64;
        let prev = self.last_pointer.insert(pointer, new_last_pointer);
        if let Some(prev_pointer) = prev {
            self.go_to(prev_pointer).await?;
            AsyncWrite::write_all(&mut self.file, &new_record_position.to_be_bytes()).await?;
        }
        Ok(())
    }

    /// Writes a declaration and and a begin stream marker.
    pub async fn write_declaration(&mut self, path: Path) -> Result<(), Error> {
        self.go_to_end().await?;
        let pointer = Pointer::Declaration;
        let record = Record::Declaration { path };
        self.write_record(pointer, &record).await?;
        Ok(())
    }

    async fn read_record(&mut self, pos: u64) -> Result<(Record, Option<u64>), Error> {
        log::trace!("Reading record at: {}", pos);
        self.go_to(pos).await?;
        let size = self.file.read_u64().await? as usize;
        log::debug!("Reading record with size: {}", size);
        let mut buf = Vec::with_capacity(size);
        buf.resize(size, 0);
        self.file.read_exact(&mut buf).await?;
        let mut cursor = Cursor::new(buf);
        let document = bson::Document::from_reader(&mut cursor)?;
        let record: Record = bson::from_document(document)?;
        let next_pos = self.file.read_u64().await?;
        let next = {
            if next_pos != 0 {
                Some(next_pos)
            } else {
                None
            }
        };
        Ok((record, next))
    }

    async fn read_records(&mut self, pointer: Pointer) -> Result<Vec<Record>, Error> {
        let mut pos = 0;
        let mut records = Vec::new();
        loop {
            let (record, next) = self.read_record(pos).await?;
            records.push(record);
            if let Some(new_pos) = next {
                pos = new_pos;
            } else {
                break;
            }
        }
        Ok(records)
    }

    async fn flush(&mut self) -> Result<(), Error> {
        self.file.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_storage_declarations() -> Result<(), Error> {
        env_logger::try_init()?;
        let file = NamedTempFile::new()?;
        let tmp_path = file.into_temp_path();

        let mut log_file = LogFile::open(&tmp_path).await?;

        let path_1: Path = "my.path.one".parse()?;
        log_file.write_declaration(path_1.clone()).await?;

        let path_2: Path = "my.path.two".parse()?;
        log_file.write_declaration(path_2.clone()).await?;

        let records = log_file.read_records(Pointer::Declaration).await?;
        assert_eq!(records.len(), 2);
        assert_eq!(
            records,
            vec![
                Record::Declaration { path: path_1 },
                Record::Declaration { path: path_2 },
            ]
        );

        tmp_path.close()?;
        Ok(())
    }
}
