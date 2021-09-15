use super::{Group, RillPool};
use crate::distributor::ParcelDistributor;
use anyhow::Error;
use async_trait::async_trait;
use meio::{
    Consumer, Context, IdOf, InstantAction, InstantActionHandler, LiteTask, Parcel, TaskEliminated,
    TaskError,
};
use once_cell::sync::Lazy;
use thiserror::Error;

pub(crate) static DISTRIBUTOR: Lazy<ParcelDistributor<RillPool>> =
    Lazy::new(ParcelDistributor::new);

impl RillPool {
    pub(super) async fn attach_distributor(
        &mut self,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let rx = DISTRIBUTOR.take_receiver().await?;
        ctx.attach(rx, (), Group::ParcelStream);
        Ok(())
    }

    pub(super) fn detach_distributor(&mut self) {
        DISTRIBUTOR.sender.close_channel();
        // NEVER terminate the group. The channel above has to be drained!!!
        //ctx.terminate_group(Group::ParcelStream);
    }
}

#[async_trait]
impl Consumer<Parcel<Self>> for RillPool {
    async fn handle(&mut self, parcel: Parcel<Self>, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.address().unpack_parcel(parcel)
    }

    async fn finished(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl<T: RillPoolTask> InstantActionHandler<AttachTask<T>> for RillPool {
    async fn handle(&mut self, msg: AttachTask<T>, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.spawn_task(msg, (), Group::Tasks);
        Ok(())
    }
}

#[async_trait]
impl<T: RillPoolTask> TaskEliminated<AttachTask<T>, ()> for RillPool {
    async fn handle(
        &mut self,
        _id: IdOf<AttachTask<T>>,
        _tag: (),
        _result: Result<(), TaskError>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

pub(crate) struct AttachTask<T> {
    task: T,
}

impl<T: RillPoolTask> InstantAction for AttachTask<T> {}

#[async_trait]
impl<T: RillPoolTask> LiteTask for AttachTask<T> {
    type Output = ();

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        self.task.routine().await
    }
}

#[derive(Error, Debug)]
#[error("Task not spawned, because pool has not started (channel lost or not exists).")]
pub struct TaskNotSpawned;

impl ParcelDistributor<RillPool> {
    pub fn spawn_task<T>(&self, task: T) -> Result<(), TaskNotSpawned>
    where
        T: RillPoolTask,
    {
        let msg = AttachTask { task };
        let parcel = Parcel::pack(msg);
        self.sender
            .unbounded_send(parcel)
            .map_err(|_| TaskNotSpawned)
    }
}

#[async_trait]
pub trait RillPoolTask: Send + 'static {
    async fn routine(self) -> Result<(), Error>;
}
