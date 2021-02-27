use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::marker::PhantomData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T: Origin, D> {
    pub direct_id: DirectId<T>,
    pub data: D,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WideEnvelope<T: Origin, D> {
    pub direction: Direction<T>,
    pub data: D,
}

/// The origin of `DirectId`.
pub trait Origin: Default + Clone {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DirectId<T: Origin> {
    value: u64,
    origin: PhantomData<T>,
}

impl<T: Origin> From<usize> for DirectId<T> {
    fn from(value: usize) -> Self {
        Self {
            // TODO: TryInto
            value: value as u64,
            origin: PhantomData,
        }
    }
}

impl<T: Origin> Into<usize> for DirectId<T> {
    fn into(self) -> usize {
        // TODO: TryInto
        self.value as usize
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Direction<T: Origin> {
    Direct(DirectId<T>),
    Multicast(Vec<DirectId<T>>),
    Broadcast,
}

impl<T: Origin> Direction<T> {
    pub fn into_vec(self) -> Vec<DirectId<T>> {
        match self {
            Self::Direct(direct_id) => vec![direct_id],
            Self::Multicast(ids) => ids,
            Self::Broadcast => Vec::new(),
        }
    }
}

impl<T: Origin> Direction<T> {
    pub fn broadcast() -> Self {
        Self::Broadcast
    }
}

impl<T: Origin> From<&HashSet<DirectId<T>>> for Direction<T> {
    fn from(set: &HashSet<DirectId<T>>) -> Self {
        let mut iter = set.iter();
        match iter.len() {
            0 => Self::Broadcast,
            1 => {
                let direct_id = iter.next().cloned().unwrap();
                Self::Direct(direct_id)
            }
            _ => {
                let ids = iter.cloned().collect();
                Self::Multicast(ids)
            }
        }
    }
}

impl<T: Origin> From<DirectId<T>> for Direction<T> {
    fn from(direct_id: DirectId<T>) -> Self {
        Self::Direct(direct_id)
    }
}
