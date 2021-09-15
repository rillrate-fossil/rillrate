use super::state::{DashboardState, ResolvedGroup, PATHS};
use anyhow::Error;
use rate_ui::shared_object::{DataChanged, SharedObject};
use rate_ui::widget::{Context, NotificationHandler, Widget, WidgetRuntime};
use rill_protocol::io::provider::EntryId;
use yew::{html, Html};

pub type Dashboard = WidgetRuntime<DashboardWidget>;

pub struct DashboardWidget {
    paths: SharedObject<DashboardState>,
}

impl Default for DashboardWidget {
    fn default() -> Self {
        Self {
            paths: PATHS.with(SharedObject::clone),
        }
    }
}

impl Widget for DashboardWidget {
    type Event = ();
    type Tag = ();
    type Properties = ();
    type Meta = ();

    fn init(&mut self, ctx: &mut Context<Self>) {
        self.paths.subscribe(ctx);
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let state = self.paths.read();
        let dashboard = state.get_dashboard().unwrap_or_default();
        let has_visuals = !dashboard.visuals.is_empty();
        let has_controls = !dashboard.controls.is_empty();
        let visuals = {
            if has_visuals {
                let style = "";
                html! {
                    <div class="flex-grow-1" style=style>
                        { for dashboard.visuals.into_iter().map(render_group) }
                    </div>
                }
            } else {
                Html::default()
            }
        };
        let controls = {
            if has_controls {
                let style = "width: 340px;";
                html! {
                    <div style=style>
                        { for dashboard.controls.into_iter().map(render_group) }
                    </div>
                }
            } else {
                Html::default()
            }
        };
        let transparents = {
            html! {
                <div>
                    { for dashboard.transparents.into_iter().map(render_group) }
                </div>
            }
        };
        if has_visuals || has_controls {
            html! {
                <div yew=module_path!() class="d-flex flex-column">
                    <div class="d-flex flex-column flex-md-row">
                        { visuals }
                        { controls }
                    </div>
                    { transparents }
                </div>
            }
        } else {
            html! {
                <div class="container text-center mt-5">
                    <p class="fs-4">{ "Streams are not available, or the app is disconnected." }</p>
                    <p class="fs-3 mt-3">
                        //{ "Welcome to the "}
                        <span class="fw-bold">
                            { " RillRate " }
                            <span class="text-danger">{ " Live " }</span>
                        </span>
                        //{" embedded dashboard" }
                    </p>
                    //{ for state.structure.packages.iter().map(render_package) }
                </div>
            }
        }
    }
}

fn render_group((name, streams): (EntryId, ResolvedGroup)) -> Html {
    html! {
        <div yew="render_group">
            <div class="text-primary p-2">{ name }</div>
            <super::GroupViewer streams=streams />
        </div>
    }
}

impl NotificationHandler<DataChanged<DashboardState>> for DashboardWidget {
    fn handle(
        &mut self,
        _event: DataChanged<DashboardState>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.redraw();
        Ok(())
    }
}
