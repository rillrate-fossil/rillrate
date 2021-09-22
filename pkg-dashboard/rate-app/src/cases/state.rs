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

// TODO: Move Out
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
    // TODO: Move to `CaseStructure` struct
    pub layouts: BTreeMap<EntryId, Layout>,

    // TODO: Move Out
    pub global_scene: GlobalScene,

    // TODO: Move to `CaseSelection` struct
    pub selected_layout: Option<EntryId>,
    pub selected_tab: Option<EntryId>,
}

impl SceneState {
    fn autoselect(&mut self) -> Option<()> {
        let mut layout = self.selected_layout.clone().unwrap_or_default();
        let mut tab = self.selected_tab.clone().unwrap_or_default();
        let layouts = &self.layouts;
        if !layouts.contains_key(&layout) {
            layout = layouts.keys().next().cloned()?;
        }
        let tabs = &layouts.get(&layout)?.tabs;
        if !tabs.contains_key(&tab) {
            tab = tabs.keys().next().cloned()?;
        }
        self.selected_layout = Some(layout);
        self.selected_tab = Some(tab);
        Some(())
    }
}

// TODO: Implement auto-select

impl SceneState {
    pub fn get_layout_tab(&self) -> Option<&LayoutTab> {
        let selected_layout = self.selected_layout.as_ref()?;
        let selected_tab = self.selected_tab.as_ref()?;
        let layout = self.layouts.get(selected_layout)?;
        let tab = layout.tabs.get(selected_tab)?;
        Some(tab)
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
