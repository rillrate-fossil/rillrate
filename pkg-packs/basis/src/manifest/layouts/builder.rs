use super::components::{Element, Layout};
use crate::manifest::layouts::global::LAYOUTS;
use crate::paths::LayoutPath;

impl Layout {
    pub fn new(name: impl Into<LayoutPath>) -> Self {
        Self {
            name: name.into().into(),
            element: Element::Empty,
        }
    }

    pub fn set_container(&mut self, element: impl Into<Element>) {
        self.element = element.into();
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
