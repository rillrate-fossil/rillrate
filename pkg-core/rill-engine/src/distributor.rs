use futures::channel::mpsc;
use futures::lock::Mutex;
use meio::{Actor, Parcel};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Reserved receiver already taken.")]
pub struct AlreadyTaken;

pub(crate) struct ParcelDistributor<A: Actor> {
    pub sender: mpsc::UnboundedSender<Parcel<A>>,
    pub receiver: Mutex<Option<mpsc::UnboundedReceiver<Parcel<A>>>>,
}

impl<A: Actor> ParcelDistributor<A> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded();
        let receiver = Mutex::new(Some(rx));
        Self {
            sender: tx,
            receiver,
        }
    }

    pub async fn take_receiver(&self) -> Result<mpsc::UnboundedReceiver<Parcel<A>>, AlreadyTaken> {
        self.receiver.lock().await.take().ok_or(AlreadyTaken)
    }
}
