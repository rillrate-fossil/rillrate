use yew::{html, Children, Component, ComponentLink, Html, Properties, ShouldRender};

pub struct LabeledForm {
    props: Props,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub label: &'static str,
    pub children: Children,
}

impl Component for LabeledForm {
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
            <div class="d-flex flex-column flex-grow-1 justify-content-start align-items-center">
                <div class="container my-5 py-5"></div>
                <div class="form form-labeled mb-5 text-center">
                    <h3 class="mb-3">{ "RillRate" }<span class="fw-normal ms-2">{ &self.props.label }</span></h3>
                    { self.props.children.clone() }
                    <p class="mt-4 text-muted">{ "© 2021 RillRate OÜ" }</p>
                </div>
            </div>
        }
    }
}
