//use crate::common::auth;
use crate::widget::{Context, Widget, WidgetRuntime};
use yew::{html, Html, Properties};

pub type Header = WidgetRuntime<HeaderWidget>;

#[derive(Default)]
pub struct HeaderWidget;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub label: &'static str,
    //pub children: Children,
}

impl Widget for HeaderWidget {
    type Event = ();
    type Tag = ();
    type Properties = Props;
    type Meta = ();
    type RouterState = ();

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <header class="d-flex flex-column flex-md-row align-items-center p-3 px-md-4 bg-white border-bottom shadow-sm">
                <p class="h5 my-0 me-md-2 fw-bold">{ "RillRate" }</p>
                <p class="h5 my-0 me-md-5 fw-normal">{ ctx.properties().label }</p>
                // TODO: Or use space-between at this level and put children here?
                <div class="d-flex mx-auto">
                    // TODO: Put children here? Or see above?
                </div>
                //<auth::SignOut />
                //<a class="btn btn-outline-primary" onclick=ctx.event(Msg::DoSignOut)>{ "Sign Out" }</a>
            </header>
        }
    }
}
