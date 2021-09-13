use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

pub fn spinner(reason: &'static str) -> Html {
    html! {
        <Spinner reason=reason />
    }
}

#[derive(Debug, Properties, Clone)]
pub struct Props {
    pub reason: &'static str,
}

pub struct Spinner {
    reason: &'static str,
}

impl Component for Spinner {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            reason: props.reason,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div yew=module_path!() class="flex-grow-1 d-flex justify-content-center align-items-center">
                <div class="spinner-grow text-primary">
                    <span class="visually-hidden">{ self.reason }</span>
                </div>
            </div>
        }
    }
}
