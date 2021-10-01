use super::components::Container;
use super::layout::*;
use crate::manifest::layouts::global::LAYOUTS;
use crate::paths::LayoutPath;

impl Layout {
    pub fn new(name: impl Into<LayoutPath>) -> Self {
        Self {
            name: name.into().into(),
            container: Container::Empty,
        }
    }

    pub fn set_container(&mut self, container: impl Into<Container>) {
        self.container = container.into();
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
