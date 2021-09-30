use super::components::Container;
use super::layout::*;
use crate::manifest::layouts::global::LAYOUTS;
use crate::paths::{AutoPath, LayoutPath};
use rill_protocol::io::provider::Path;

impl Layout {
    pub fn new(name: impl Into<LayoutPath>) -> Self {
        Self {
            name: name.into().into(),
            container: Container::Empty,
            items: Vec::new(),
            labels: Vec::new(),
        }
    }

    pub fn set_container(&mut self, container: impl Into<Container>) {
        self.container = container.into();
    }

    pub fn add_item(
        &mut self,
        position: impl Into<Position>,
        size: impl Into<Size>,
        path: impl Into<AutoPath>,
    ) {
        let item = LayoutItem {
            position: position.into(),
            size: size.into(),
            path: Path::from(path.into()),
        };
        self.items.push(item);
    }

    pub fn add_label(
        &mut self,
        position: impl Into<Position>,
        size: impl Into<Size>,
        text: impl Into<String>,
    ) {
        let item = Label {
            position: position.into(),
            size: size.into(),
            text: text.into(),
        };
        self.labels.push(item);
    }

    pub fn register(&self) {
        let name = self.name.clone();
        let layout = self.clone();
        LAYOUTS.add_tab(name, layout);
    }

    pub fn unregister(&self) {
        let name = self.name.clone();
        LAYOUTS.remove_tab(name);
    }
}
