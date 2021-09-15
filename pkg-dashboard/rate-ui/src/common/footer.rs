use crate::widget::{Context, Widget, WidgetRuntime};
use yew::{html, Html};

pub type Footer = WidgetRuntime<FooterWidget>;

#[derive(Default)]
pub struct FooterWidget;

impl Widget for FooterWidget {
    type Event = ();
    type Tag = ();
    type Properties = ();
    type Meta = ();
    type RouterState = ();

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let dev = if cfg!(debug_assertions) { "-dev" } else { "" };
        html! {
            <footer class="footer text-muted bg-white border-top">
                <div class="container-fluid">
                    <div class="d-flex justify-content-between mt-3">
                        <p>{ "© 2021 RillRate OÜ" }</p>
                        <p class="fw-light">{ format!("v{}{}", crate::meta::VERSION, dev) }</p>
                    </div>
                </div>
            </footer>
        }
    }
}
