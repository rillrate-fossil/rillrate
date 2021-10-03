use super::record::Record;
use crate::explorer::state::PATHS;
use rate_ui::shared_object::SharedObject;
use yew::{html, Html};

pub trait LayoutRender {
    fn layout_render(&self) -> Html;
}

pub trait ToStyle {
    fn to_style(&self) -> &'static str;
}

use rrpack_basis::manifest::layouts::components::Element;

impl LayoutRender for Element {
    fn layout_render(&self) -> Html {
        match self {
            Self::Empty => {
                html! {}
            }
            Self::Align(value) => value.layout_render(),
            Self::Center(value) => value.layout_render(),
            Self::Container(value) => value.layout_render(),
            Self::Expanded(value) => value.layout_render(),
            Self::Spacer(value) => value.layout_render(),
            Self::Row(value) => value.layout_render(),
            Self::Column(value) => value.layout_render(),

            Self::Text(value) => value.layout_render(),
            Self::Flow(value) => value.layout_render(),
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

use rrpack_basis::manifest::layouts::components::Center;

impl LayoutRender for Center {
    fn layout_render(&self) -> Html {
        html! {
            <div yew="Center" class="d-flex flex-column justify-content-center align-items-center">
                { self.child.layout_render() }
            </div>
        }
    }
}

// TODO: Consider removing the `Container`
use rrpack_basis::manifest::layouts::components::Container;

impl LayoutRender for Container {
    fn layout_render(&self) -> Html {
        html! {
            <div yew="Container" class="d-flex flex-column" style="flex-grow: 1;">
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

use rrpack_basis::manifest::layouts::components::Spacer;

impl LayoutRender for Spacer {
    fn layout_render(&self) -> Html {
        let style = format!("flex-grow: {};", self.flex);
        let class = if self.maintenance { "maintenance" } else { "" };
        html! {
            <div yew="Spacer" class=class style=style></div>
        }
    }
}

use rrpack_basis::manifest::layouts::components::Row;

impl LayoutRender for Row {
    fn layout_render(&self) -> Html {
        html! {
            <div yew="Row" class="d-flex flex-row rr-stretched">
                { for self.children.iter().map(LayoutRender::layout_render) }
            </div>
        }
    }
}

use rrpack_basis::manifest::layouts::components::Column;

impl LayoutRender for Column {
    fn layout_render(&self) -> Html {
        html! {
            <div yew="Column" class="d-flex flex-column rr-stretched">
                { for self.children.iter().map(LayoutRender::layout_render) }
            </div>
        }
    }
}

use rrpack_basis::manifest::layouts::components::Text;

impl LayoutRender for Text {
    fn layout_render(&self) -> Html {
        html! {
            <div yew="Text" class="rr-component" style=self.align.to_style()>{ &self.text }</div>
        }
    }
}

use rrpack_basis::manifest::layouts::components::TextAlign;

impl ToStyle for TextAlign {
    fn to_style(&self) -> &'static str {
        match self {
            TextAlign::Left => "text-align: left;",
            TextAlign::Right => "text-align: right;",
            TextAlign::Center => "text-align: center;",
            TextAlign::Justify => "text-align: justify;",
            TextAlign::Start => "text-align: start;",
            TextAlign::End => "text-align: end;",
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
