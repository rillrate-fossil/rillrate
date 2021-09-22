use rate_ui::shared_object::{RouterState, SharedObject};
use rate_ui::storage::typed_storage::Storable;
use rill_protocol::io::provider::EntryId;
use rrpack_basis::manifest::layouts::layout::{Layout, LayoutTab};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use strum::{Display, EnumIter};

thread_local! {
    pub static SCENE: SharedObject<SceneState> = SharedObject::new();
}

#[derive(EnumIter, Display, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum GlobalScene {
    Home,
    Cases,
    Explorer,
}

impl Default for GlobalScene {
    fn default() -> Self {
        Self::Home
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct SceneState {
    pub layouts: BTreeMap<EntryId, Layout>,
    pub selected_layout: Option<EntryId>,
    pub selected_tab: Option<EntryId>,
    pub global_scene: GlobalScene,
}

impl SceneState {
    fn autoselect(&mut self) {
        if self.selected_layout.is_none() {
            self.selected_layout = self.layouts.keys().next().cloned();
        }
    }
}

// TODO: Implement auto-select

impl SceneState {
    pub fn get_layout(&self) -> Option<&Layout> {
        self.selected_layout
            .as_ref()
            .and_then(|id| self.layouts.get(id))
    }

    pub fn get_layout_tab(&self) -> Option<&LayoutTab> {
        None
    }
}

impl Storable for SceneState {
    fn key() -> &'static str {
        module_path!()
    }
}

impl RouterState for SceneState {
    fn restored(&mut self) {
        self.global_scene = Default::default();
        self.layouts.clear();
    }

    fn on_update(&mut self) {
        self.autoselect();
    }
}
