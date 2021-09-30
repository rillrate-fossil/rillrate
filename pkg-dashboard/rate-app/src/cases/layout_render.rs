use super::record::Record;
use crate::explorer::state::PATHS;
use rate_ui::shared_object::SharedObject;
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
            Self::Expanded(value) => value.layout_render(),
            Self::Row(value) => value.layout_render(),
            Self::Column(value) => value.layout_render(),
        }
    }
}

use rrpack_basis::manifest::layouts::components::Align;

impl LayoutRender for Align {
    fn layout_render(&self) -> Html {
        html! {
            <div yew="Align">
                { self.child.layout_render() }
            </div>
        }
    }
}

use rrpack_basis::manifest::layouts::components::Expanded;

impl LayoutRender for Expanded {
    fn layout_render(&self) -> Html {
        let style = format!("flex-grow: {};", self.flex);
        html! {
            <div yew="Expanded" style=style>
                { self.child.layout_render() }
            </div>
        }
    }
}

use rrpack_basis::manifest::layouts::components::Row;

impl LayoutRender for Row {
    fn layout_render(&self) -> Html {
        html! {
            <div yew="Row" class="d-flex flex-row">
                { for self.children.iter().map(LayoutRender::layout_render) }
            </div>
        }
    }
}

use rrpack_basis::manifest::layouts::components::Column;

impl LayoutRender for Column {
    fn layout_render(&self) -> Html {
        html! {
            <div yew="Column" class="d-flex flex-column">
                { for self.children.iter().map(LayoutRender::layout_render) }
            </div>
        }
    }
}

use rrpack_basis::manifest::layouts::components::Element;

impl LayoutRender for Element {
    fn layout_render(&self) -> Html {
        match self {
            Self::Container(value) => value.layout_render(),
            Self::Spacer(value) => value.layout_render(),
            Self::Label(value) => value.layout_render(),
            Self::Flow(value) => value.layout_render(),
        }
    }
}

use rrpack_basis::manifest::layouts::components::Spacer;

impl LayoutRender for Spacer {
    fn layout_render(&self) -> Html {
        html! {
            <div yew="Spacer"></div>
        }
    }
}

use rrpack_basis::manifest::layouts::components::Label;

impl LayoutRender for Label {
    fn layout_render(&self) -> Html {
        html! {
            <div yew="Label">{ &self.text }</div>
        }
    }
}

use rrpack_basis::manifest::layouts::components::Flow;

impl LayoutRender for Flow {
    fn layout_render(&self) -> Html {
        let paths = PATHS.with(SharedObject::clone);
        let paths = paths.read();
        if let Some(desc) = paths.descs.get(&self.path) {
            Record::from(desc).render()
        } else {
            html! {}
        }
    }
}
