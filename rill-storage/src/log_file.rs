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
        // Always writes to the end
        self.go_to_end().await?;
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

    /// Reads record at the specific position
    async fn read_record(&mut self, pos: u64) -> Result<(Record, Option<u64>), Error> {
        log::trace!("Reading record at: {}", pos);
        self.go_to(pos).await?;
        // Reads size of serialized data
        let size = self.file.read_u64().await? as usize;
        // Prepare a buffer to absorb that data
        log::debug!("Reading record with size: {}", size);
        let mut buf = Vec::with_capacity(size);
        buf.resize(size, 0);
        self.file.read_exact(&mut buf).await?;
        // Deserializing
        let mut cursor = Cursor::new(buf);
        let document = bson::Document::from_reader(&mut cursor)?;
        let record: Record = bson::from_document(document)?;
        // Reading position of the next record
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
        let mut rake = false;
        loop {
            let (record, next) = self.read_record(pos).await?;
            if rake {
                records.push(record);
            } else {
                match &pointer {
                    Pointer::Declaration => {
                        records.push(record);
                        rake = true;
                    }
                    Pointer::Event { path: event_path } => {
                        if let Record::Declaration { path: record_path } = &record {
                            if event_path == record_path {
                                rake = true;
                                pos = self.get_position().await?;
                                // Don't follow the next field of declarations
                                // Read the next record
                                continue;
                            }
                        }
                    }
                }
            }
            if let Some(new_pos) = next {
                pos = new_pos;
            } else {
                break;
            }
        }
        log::trace!("Records found: {:?}", records);
        Ok(records)
    }

    pub async fn write_event(&mut self, path: &Path, event: RillEvent) -> Result<(), Error> {
        let key = Pointer::Event { path: path.clone() };
        if !self.last_pointer.contains_key(&key) {
            // Writes a declaration
            let pointer = Pointer::Declaration;
            let record = Record::Declaration { path: path.clone() };
            self.write_record(pointer, &record).await?;
        }
        let record = Record::Event { event };
        self.write_record(key, &record).await?;
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), Error> {
        self.file.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rill_protocol::provider::{RillData, Timestamp};
    use tempfile::NamedTempFile;

    fn now() -> Timestamp {
        use std::time::SystemTime;
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .into()
    }

    #[tokio::test]
    async fn test_storage_declarations() -> Result<(), Error> {
        env_logger::try_init()?;
        let file = NamedTempFile::new()?;
        let tmp_path = file.into_temp_path();

        let mut log_file = LogFile::open(&tmp_path).await?;

        let path_1: Path = "my.path.one".parse()?;
        let event_1 = RillEvent {
            timestamp: now(),
            data: RillData::CounterRecord { value: 10.0 },
        };
        log_file.write_event(&path_1, event_1.clone()).await?;

        let path_2: Path = "my.path.two".parse()?;
        let event_2 = RillEvent {
            timestamp: now(),
            data: RillData::CounterRecord { value: 20.0 },
        };
        log_file.write_event(&path_2, event_2.clone()).await?;

        let records = log_file.read_records(Pointer::Declaration).await?;
        assert_eq!(records.len(), 2);
        assert_eq!(
            records,
            vec![
                Record::Declaration {
                    path: path_1.clone()
                },
                Record::Declaration {
                    path: path_2.clone()
                },
            ]
        );

        let records = log_file
            .read_records(Pointer::Event { path: path_1 })
            .await?;
        assert_eq!(records.len(), 1);
        assert_eq!(records, vec![Record::Event { event: event_1 },]);

        let records = log_file
            .read_records(Pointer::Event { path: path_2 })
            .await?;
        assert_eq!(records.len(), 1);
        assert_eq!(records, vec![Record::Event { event: event_2 },]);

        tmp_path.close()?;
        Ok(())
    }
}
