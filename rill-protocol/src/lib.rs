pub mod config;
pub mod flow;
pub mod frame;
pub mod io;
pub mod pathfinder;
pub mod range;

pub mod encoding {
    use anyhow::Error;
    use serde::{Deserialize, Serialize};

    pub fn from_slice<'a, T>(v: &'a [u8]) -> Result<T, Error>
    where
        T: Deserialize<'a>,
    {
        //bincode::deserialize(v).map_err(Error::from)
        flexbuffers::from_slice(v).map_err(Error::from)
        //serde_json::from_slice(v).map_err(Error::from)
    }

    pub fn to_vec<T: ?Sized>(value: &T) -> Result<Vec<u8>, Error>
    where
        T: Serialize,
    {
        //bincode::serialize(value).map_err(Error::from)
        flexbuffers::to_vec(value).map_err(Error::from)
        //serde_json::to_vec(value).map_err(Error::from)
    }
}

metacrate::meta!();
