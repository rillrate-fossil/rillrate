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
            Self::Align(value) => value.layout_render(),
            Self::Row(value) => value.layout_render(),
            Self::Column(value) => value.layout_render(),
        }
    }
}

use rrpack_basis::manifest::layouts::components::Align;

impl LayoutRender for Align {
    fn layout_render(&self) -> Html {
        html! {
            <div>
                { self.child.layout_render() }
            </div>
        }
    }
}

use rrpack_basis::manifest::layouts::components::Row;

impl LayoutRender for Row {
    fn layout_render(&self) -> Html {
        html! {
            <div class="d-flex flex-row">
                { for self.children.iter().map(LayoutRender::layout_render) }
            </div>
        }
    }
}

use rrpack_basis::manifest::layouts::components::Column;

impl LayoutRender for Column {
    fn layout_render(&self) -> Html {
        html! {
            <div class="d-flex flex-column">
                { for self.children.iter().map(LayoutRender::layout_render) }
            </div>
        }
    }
}

use rrpack_basis::manifest::layouts::components::Element;

impl LayoutRender for Element {
    fn layout_render(&self) -> Html {
        match self {
            Self::Label(value) => value.layout_render(),
        }
    }
}

use rrpack_basis::manifest::layouts::components::Label;

impl LayoutRender for Label {
    fn layout_render(&self) -> Html {
        html! {
            <div>{ &self.text }</div>
        }
    }
}
