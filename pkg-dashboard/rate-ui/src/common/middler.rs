use yew::{html, Children, Component, ComponentLink, Html, Properties, ShouldRender};

pub struct Middler {
    props: Props,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub children: Children,
}

impl Component for Middler {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="d-flex flex-grow-1 align-items-center justify-content-center">
                <div class="text-center">
                    { self.props.children.clone() }
                </div>
            </div>
        }
    }
}
