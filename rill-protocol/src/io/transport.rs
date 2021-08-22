use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;

/// An `Envelope` with service-layer messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceEnvelope<T: Origin, D, S> {
    Service(S),
    Envelope(Envelope<T, D>),
}

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
pub trait Origin: Default + Clone + PartialEq + Eq + Hash {}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DirectId<T: Origin> {
    value: u64,
    origin: PhantomData<T>,
}

impl<T: Origin> fmt::Debug for DirectId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DirectId").field(&self.value).finish()
    }
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

impl<T: Origin> From<DirectId<T>> for usize {
    fn from(this: DirectId<T>) -> usize {
        // TODO: TryInto
        this.value as usize
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Direction<T: Origin> {
    Direct(DirectId<T>),
    Multicast(HashSet<DirectId<T>>),
    // TODO: Remove, since all streames bootstrapped from
    // a predefined path the Broadcast direction is not needed anymore.
    Broadcast,
}

impl<T: Origin> Direction<T> {
    pub fn into_vec(self) -> Vec<DirectId<T>> {
        match self {
            Self::Direct(direct_id) => vec![direct_id],
            Self::Multicast(ids) => ids.into_iter().collect(),
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
