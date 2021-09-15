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

pub fn pack<T: ?Sized, P: From<Vec<u8>>>(value: &T) -> Result<P, Error>
where
    T: Serialize,
{
    flexbuffers::to_vec(value).map_err(Error::from).map(P::from)
}

pub fn unpack<T, P>(v: T) -> Result<P, Error>
where
    T: AsRef<[u8]>,
    P: for<'a> Deserialize<'a>,
{
    let data = v.as_ref();
    flexbuffers::from_slice(data).map_err(Error::from)
}
