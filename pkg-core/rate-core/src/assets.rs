use anyhow::Error;
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use tar::Archive;
use thiserror::Error;

#[derive(Debug, Error)]
enum Reason {
    #[error("wrong format")]
    WrongFormat,
}

#[derive(Clone, Debug)]
pub struct Assets {
    files: Arc<HashMap<String, Vec<u8>>>,
}

impl Assets {
    /// Expected gzipped tar file contents.
    pub fn parse(assets: &[u8]) -> Result<Assets, Error> {
        let tar = GzDecoder::new(assets);
        let mut archive = Archive::new(tar);
        let mut files = HashMap::new();
        for entry in archive.entries()? {
            let mut entry = entry?;
            let mut data = Vec::new();
            entry.read_to_end(&mut data)?;
            if !data.is_empty() {
                let name = entry
                    .path()?
                    .to_str()
                    .ok_or(Reason::WrongFormat)?
                    .to_owned();
                #[cfg(debug_assertions)]
                log::trace!("Register asset file: {}", name);
                files.insert(name, data);
            }
        }
        Ok(Self {
            files: Arc::new(files),
        })
    }

    pub fn get(&self, path: &str) -> Option<&[u8]> {
        self.files.get(path).map(Vec::as_ref)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.files.iter().map(|(s, d)| (s.as_ref(), d.as_ref()))
    }
}
