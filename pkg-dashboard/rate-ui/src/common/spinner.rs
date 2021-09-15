use yew::{html, Component, ComponentLink, Html, ShouldRender};

pub struct Spinner;

impl Component for Spinner {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="flex-grow-1 d-flex justify-content-center align-items-center">
                <div class="spinner-grow text-primary">
                    <span class="visually-hidden">{ "Loading..." }</span>
                </div>
            </div>
        }
    }
}
