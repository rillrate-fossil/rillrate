use super::{Group, Router};
use crate::actors::provider_session::ProviderSession;
use crate::actors::supervisor::Supervisor;
use crate::info::TRACERS;
use anyhow::Error;
use async_trait::async_trait;
use meio::{ActionHandler, Context, Eliminated, IdOf, InteractionHandler};
use meio_connect::hyper::{Body, Response};
use meio_connect::server::{DirectPath, NoParameters, Req, WebRoute, WsReq, WsRoute};
use rill_protocol::io::provider::ProviderProtocol;

impl<T: Supervisor> Router<T> {
    pub(super) async fn init_internal(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let route = WebRoute::new(Index, ctx.address().clone());
        self.internal_server.add_route(route).await?;

        let route = WsRoute::new(ProviderLive, ctx.address().clone());
        self.internal_server.add_route(route).await?;

        Ok(())
    }
}

struct Index;

impl DirectPath for Index {
    type Output = NoParameters;
    type Parameter = ();
    fn paths() -> &'static [&'static str] {
        &["/", "/index.html"]
    }
}

const SMART_REDIRECT: &str = r#"
<html>
<head>
    <script>
        window.location.port = ${PORT};
    </script>
</head>
<body>
    <p>RillRate Vision</p>
</body>
</html>
"#;

#[async_trait]
impl<T: Supervisor> InteractionHandler<Req<Index>> for Router<T> {
    async fn handle(
        &mut self,
        _: Req<Index>,
        _ctx: &mut Context<Self>,
    ) -> Result<Response<Body>, Error> {
        let redirect = SMART_REDIRECT.replace("${PORT}", &self.external_port.to_string());
        let response = Response::builder().body(redirect.into())?;
        Ok(response)
    }
}

struct ProviderLive;

impl DirectPath for ProviderLive {
    type Output = NoParameters;
    type Parameter = ProviderProtocol;

    fn paths() -> &'static [&'static str] {
        &["/live/provider"]
    }
}

#[async_trait]
impl<T: Supervisor> ActionHandler<WsReq<ProviderLive>> for Router<T> {
    async fn handle(
        &mut self,
        req: WsReq<ProviderLive>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        if !ctx.is_terminating() {
            //let ip = req.stream.addr().ip();
            if self.active_providers.has_slot() {
                let session_actor = ProviderSession::new(req.stream, self.registry.clone());
                let addr = ctx.spawn_actor(session_actor, Group::Internals);
                self.active_providers.acquire(addr);
            } else {
                let alert = format!(
                    "Active providers {} limit reached.",
                    self.active_providers.limit().total
                );
                TRACERS.alerts.alert(alert);
                log::warn!("Limit of active providers reached.");
            }
        } else {
            log::warn!("Incoming provider connection rejected, because the server is terminating.");
        }
        Ok(())
    }
}

#[async_trait]
impl<T: Supervisor> Eliminated<ProviderSession> for Router<T> {
    async fn handle(
        &mut self,
        id: IdOf<ProviderSession>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        self.active_providers.release(id);
        Ok(())
    }
}
