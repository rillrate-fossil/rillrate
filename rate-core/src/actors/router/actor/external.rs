use super::{Group, Router};
use crate::actors::client_session::ClientSession;
use crate::actors::supervisor::Supervisor;
use crate::info::TRACERS;
use anyhow::Error;
use async_trait::async_trait;
use meio::{ActionHandler, Context, Eliminated, IdOf, InteractionHandler};
use meio_connect::headers::HeaderValue;
use meio_connect::hyper::{header, Body, Response, StatusCode};
use meio_connect::server::{DirectPath, NoParameters, Req, WebRoute, WsReq, WsRoute};
use rill_protocol::io::client::ClientProtocol;

impl<T: Supervisor> Router<T> {
    pub(super) async fn init_external(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        // TODO: Make it configurable
        let route = WebRoute::new(IndexToUi, ctx.address().clone());
        self.external_server.add_route(route).await?;

        let route = WsRoute::new(ClientLive, ctx.address().clone());
        self.external_server.add_route(route).await?;

        Ok(())
    }
}

struct IndexToUi;

impl DirectPath for IndexToUi {
    type Output = NoParameters;
    type Parameter = ();
    fn paths() -> &'static [&'static str] {
        &["/", "/index.html"]
    }
}

#[async_trait]
impl<T: Supervisor> InteractionHandler<Req<IndexToUi>> for Router<T> {
    async fn handle(
        &mut self,
        _: Req<IndexToUi>,
        _ctx: &mut Context<Self>,
    ) -> Result<Response<Body>, Error> {
        let mut response = Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .body(Body::empty())?;
        // My eyes cry
        response
            .headers_mut()
            .insert(header::LOCATION, HeaderValue::from_static("/ui/"));
        Ok(response)
    }
}

struct ClientLive;

impl DirectPath for ClientLive {
    type Output = NoParameters;
    type Parameter = ClientProtocol;

    fn paths() -> &'static [&'static str] {
        &["/live/client"]
    }
}

#[async_trait]
impl<T: Supervisor> ActionHandler<WsReq<ClientLive>> for Router<T> {
    async fn handle(
        &mut self,
        req: WsReq<ClientLive>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        if !ctx.is_terminating() {
            //let ip = req.stream.addr().ip();
            if self.active_clients.has_slot() {
                let session_actor = ClientSession::new(
                    self.supervisor.clone(),
                    req.stream,
                    self.registry.clone(),
                    self.global_acl.clone(),
                );
                let addr = ctx.spawn_actor(session_actor, Group::Externals);
                self.active_clients.acquire(addr);
            } else {
                let alert = format!(
                    "Active clients {} limit reached.",
                    self.active_clients.limit().total
                );
                TRACERS.alerts.alert(alert);
                log::warn!("Limit of active clients reached.");
            }
        } else {
            log::warn!("Incoming client connection rejected, because the server is terminating.");
        }
        Ok(())
    }
}

#[async_trait]
impl<T: Supervisor> Eliminated<ClientSession<T>> for Router<T> {
    async fn handle(
        &mut self,
        id: IdOf<ClientSession<T>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        self.active_clients.release(id);
        Ok(())
    }
}
