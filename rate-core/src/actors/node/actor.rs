pub mod link;

use super::config::NodeConfig;
use crate::actors::client_session::SessionAcl;
use crate::actors::router::Router;
use crate::actors::supervisor::Supervisor;
use crate::info;
use anyhow::Error;
use async_trait::async_trait;
use derive_more::From;
use meio::{
    Actor, Address, Context, Eliminated, IdOf, InteractionDone, InterruptedBy, StartedBy, Tag,
    TaskError,
};
use meio_connect::server::{link::WaitForAddress, HttpServer, HttpServerLink};
use rill_engine::{EngineConfig, RillEngine};
use std::net::SocketAddr;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, From)]
pub struct NodeLink<T: Supervisor> {
    address: Address<Node<T>>,
}

impl<T: Supervisor> Clone for NodeLink<T> {
    fn clone(&self) -> Self {
        Self {
            address: self.address.clone(),
        }
    }
}

pub struct Node<T: Supervisor> {
    config: NodeConfig,
    external_server: Option<HttpServerLink>,
    internal_server: Option<HttpServerLink>,
    supervisor: Address<T>,
    // TODO: RouterLink here?
    router: Option<Address<Router<T>>>,
    global_acl: SessionAcl,
}

impl<T: Supervisor> Node<T> {
    pub fn new(config: NodeConfig, supervisor: Address<T>, global_acl: SessionAcl) -> Self {
        Self {
            config,
            external_server: None,
            internal_server: None,
            supervisor,
            router: None,
            global_acl,
        }
    }

    pub fn router(&mut self) -> Result<&mut Address<Router<T>>, Error> {
        self.router
            .as_mut()
            .ok_or_else(|| Error::msg("Router lost"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Group {
    App,
    External,
    Tracer,
    Internal,
    Router,
    Service,
}

#[async_trait]
impl<T: Supervisor> Actor for Node<T> {
    type GroupBy = Group;

    fn name(&self) -> String {
        "Node".into()
    }
}

#[async_trait]
impl<T: Supervisor> StartedBy<T> for Node<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(Group::iter().collect());
        log::info!("Starting the node...");

        info::TRACERS.touch();

        log::info!("Starting internal server...");
        let http_server = HttpServer::new(self.config.internal_address());
        let internal_address = ctx.spawn_actor(http_server, Group::Internal);
        let internal_link: HttpServerLink = internal_address.link();
        let wait_addr = internal_link.wait_for_address();
        ctx.track_interaction(wait_addr, Internal, Group::Service);
        self.internal_server = Some(internal_link);

        log::info!("Starting external server...");
        let http_server = HttpServer::new(self.config.external_address());
        let external_address = ctx.spawn_actor(http_server, Group::External);
        let external_link: HttpServerLink = external_address.link();
        let wait_addr = external_link.wait_for_address();
        ctx.track_interaction(wait_addr, External, Group::Service);
        self.external_server = Some(external_link.clone());

        // TODO: `Router` is not needed in the future... Look to the `AppBind`
        log::info!("Starting router...");
        let router = Router::new(
            self.supervisor.link(),
            external_address.link(),
            self.config.external_address().port(),
            internal_address.link(),
            self.global_acl.clone(),
        );
        let router_addr = ctx.spawn_actor(router, Group::Router);
        self.router = Some(router_addr);

        Ok(())
    }
}

#[async_trait]
impl<T: Supervisor> InterruptedBy<T> for Node<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Terminating the node...");
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl<T: Supervisor> Eliminated<HttpServer> for Node<T> {
    async fn handle(
        &mut self,
        _id: IdOf<HttpServer>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::info!("HttpServer finished");
        Ok(())
    }
}

#[async_trait]
impl<T: Supervisor> Eliminated<Router<T>> for Node<T> {
    async fn handle(
        &mut self,
        _id: IdOf<Router<T>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::info!("Router finished");
        Ok(())
    }
}

struct External;

impl Tag for External {}

#[async_trait]
impl<T: Supervisor> InteractionDone<WaitForAddress, External> for Node<T> {
    async fn handle(
        &mut self,
        _tag: External,
        _addr: SocketAddr,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn failed(
        &mut self,
        _tag: External,
        err: TaskError,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::error!("Can't wait for an external server: {}", err);
        ctx.shutdown();
        Ok(())
    }
}

struct Internal;

impl Tag for Internal {}

#[async_trait]
impl<T: Supervisor> InteractionDone<WaitForAddress, Internal> for Node<T> {
    async fn handle(
        &mut self,
        _tag: Internal,
        _addr: SocketAddr,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let config = EngineConfig {
            node: None,
            name: Some(self.global_acl.id().clone()),
            // TODO: Use `StreamType` from the special package
            provider_type: "server-info".into(),
        };
        let engine = RillEngine::new(config);
        ctx.spawn_actor(engine, Group::Tracer);
        Ok(())
    }

    async fn failed(
        &mut self,
        _tag: Internal,
        err: TaskError,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::error!("Can't wait for an internal server: {}", err);
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl<T: Supervisor> Eliminated<RillEngine> for Node<T> {
    async fn handle(
        &mut self,
        _id: IdOf<RillEngine>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::info!("RillEngine finished");
        Ok(())
    }
}
