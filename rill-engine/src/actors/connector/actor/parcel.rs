use super::{Group, RillConnector};
use crate::actors::recorder::Recorder;
use crate::distributor::ParcelDistributor;
use crate::tracers::tracer::TracerMode;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Consumer, Context, InstantAction, InstantActionHandler, Parcel};
use once_cell::sync::Lazy;
use rill_protocol::flow::core;
use rill_protocol::io::provider::Description;
use std::sync::Arc;
use thiserror::Error;

/// It used by tracers to register them into the state.
pub(crate) static DISTRIBUTOR: Lazy<ParcelDistributor<RillConnector>> =
    Lazy::new(ParcelDistributor::new);

impl RillConnector {
    pub(super) async fn attach_distributor(
        &mut self,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let rx = DISTRIBUTOR.take_receiver().await?;
        ctx.attach(rx, (), Group::ParcelStream);
        Ok(())
    }

    pub(super) fn detach_distributor(&mut self, ctx: &mut Context<Self>) {
        DISTRIBUTOR.sender.close_channel();
        ctx.terminate_group(Group::ParcelStream);
    }
}

#[async_trait]
impl Consumer<Parcel<Self>> for RillConnector {
    async fn handle(&mut self, parcel: Parcel<Self>, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.address().unpack_parcel(parcel)
    }

    async fn finished(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl<T: core::Flow> InstantActionHandler<RegisterTracer<T>> for RillConnector {
    async fn handle(
        &mut self,
        msg: RegisterTracer<T>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let description = msg.description;
        let path = description.path.clone();
        log::info!("Add tracer: {}", path);
        let record = self.recorders.dig(path.clone());
        if record.get_link().is_none() {
            let packed_desc = Description::clone(&description);
            let sender = self.sender.clone();
            //let link = ctx.address().link();
            let actor = Recorder::new(description, sender, msg.mode);
            let recorder = ctx.spawn_actor(actor, Group::Recorders);
            record.set_link(recorder.link());
            // Send a description that's new tracer added
            self.registered
                .insert(recorder.id().into(), packed_desc.clone());
            self.path_flow.add(path, packed_desc);
        } else {
            log::error!("Provider for {} already registered.", path);
        }
        Ok(())
    }
}

pub(crate) struct RegisterTracer<T: core::Flow> {
    pub description: Arc<Description>,
    pub mode: TracerMode<T>,
}

impl<T: core::Flow> InstantAction for RegisterTracer<T> {}

#[derive(Error, Debug)]
#[error("Tracer not registered")]
pub struct TracerNotRegistered;

impl ParcelDistributor<RillConnector> {
    pub fn register_tracer<T>(
        &self,
        description: Arc<Description>,
        mode: TracerMode<T>,
    ) -> Result<(), TracerNotRegistered>
    where
        RillConnector: InstantActionHandler<RegisterTracer<T>>,
        T: core::Flow,
    {
        let msg = RegisterTracer { description, mode };
        let parcel = Parcel::pack(msg);
        self.sender
            .unbounded_send(parcel)
            .map_err(|_| TracerNotRegistered)
    }
}
