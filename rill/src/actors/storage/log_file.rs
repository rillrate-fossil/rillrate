use anyhow::Error;
use rill_protocol::provider::Path;
use std::collections::HashMap;
use std::io::SeekFrom;
use tokio::fs::File;
use tokio::io::AsyncSeekExt;

// [declaration: <len=u64><meta><next=u64>]
// [data: <len=u64><first_record_of_declared><next=u64>] - first == BeginStream

enum Pointer {}

enum Record {
    Declaration { path: Path },
}

pub struct LogFile {
    file: File,
    last_pointer: HashMap<Pointer, usize>,
    // TODO: Use `Seek` end instead
    //cursor: usize,
}

impl LogFile {
    pub async fn open(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        // TODO: Reopen if exists? Rotate?
        let file = File::create(path).await?;
        Ok(Self {
            file,
            last_pointer: HashMap::new(),
        })
    }

    async fn skip_to_end(&mut self) -> Result<(), Error> {
        self.file.seek(SeekFrom::End(0)).await?;
        Ok(())
    }

    async fn write_record(&mut self, record: &Record) -> Result<(), Error> {
        // 1. Serialize
        // 2. Get size
        // 3. Write size
        // 4. Write data
        // 5. Store the latest position of `next` as seek current
        // 6. Fill `next` with 0u64
        todo!();
    }

    /// Writes a declaration and and a begin stream marker.
    pub async fn begin(&mut self, path: Path) -> Result<(), Error> {
        self.skip_to_end().await?;
        let record = Record::Declaration { path };
        todo!()
    }
}
