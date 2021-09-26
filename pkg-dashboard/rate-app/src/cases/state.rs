use rate_ui::shared_object::{RouterState, SharedObject};
use rate_ui::storage::typed_storage::Storable;
use rill_protocol::io::provider::EntryId;
use rrpack_basis::manifest::layouts::layout::{Layout, LayoutTab};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

thread_local! {
    pub static CASES: SharedObject<CasesState> = SharedObject::new();
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct CasesState {
    // TODO: Move to `CaseStructure` struct
    pub layouts: BTreeMap<EntryId, Layout>,

    // TODO: Move to `CaseSelection` struct
    pub selected_layout: Option<EntryId>,
    pub selected_tab: Option<EntryId>,
}

impl CasesState {
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

impl CasesState {
    pub fn get_layout_tab(&self) -> Option<&LayoutTab> {
        let selected_layout = self.selected_layout.as_ref()?;
        let selected_tab = self.selected_tab.as_ref()?;
        let layout = self.layouts.get(selected_layout)?;
        let tab = layout.tabs.get(selected_tab)?;
        Some(tab)
    }
}

impl Storable for CasesState {
    fn key() -> &'static str {
        module_path!()
    }
}

impl RouterState for CasesState {
    fn restored(&mut self) {
        self.layouts.clear();
    }

    fn on_update(&mut self) {
        self.autoselect();
    }
}
