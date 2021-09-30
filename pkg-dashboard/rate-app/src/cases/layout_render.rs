use yew::{html, Html};

pub trait LayoutRender {
    fn layout_render(&self) -> Html;
}

use rrpack_basis::manifest::layouts::components::Container;

impl LayoutRender for Container {
    fn layout_render(&self) -> Html {
        match self {
            Self::Empty => {
                html! {}
            }
            Self::Align(align) => align.layout_render(),
        }
    }
}

use rrpack_basis::manifest::layouts::components::Align;

impl LayoutRender for Align {
    fn layout_render(&self) -> Html {
        html! {
            <p>{ "Align" }</p>
        }
    }
}
