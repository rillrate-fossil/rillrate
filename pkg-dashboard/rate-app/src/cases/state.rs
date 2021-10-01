use rate_ui::shared_object::{RouterState, SharedObject};
use rate_ui::storage::typed_storage::Storable;
use rill_protocol::io::provider::{EntryId, Path};
use rrpack_basis::manifest::layouts::components::Layout;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

thread_local! {
    pub static CASES: SharedObject<CasesState> = SharedObject::new();
}

pub type Layouts = BTreeMap<EntryId, Tabs>;
pub type Tabs = BTreeSet<EntryId>;

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct CasesStructure {
    pub layouts: Layouts,
}

impl CasesStructure {
    fn clear(&mut self) {
        self.layouts.clear();
    }

    pub fn get_packages(&self) -> impl Iterator<Item = &EntryId> {
        self.layouts.keys()
    }

    pub fn get_dashboards(&self, layout: &EntryId) -> impl Iterator<Item = &EntryId> {
        self.layouts
            .get(layout)
            .map(BTreeSet::iter)
            .into_iter()
            .flatten()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct CasesState {
    pub structure: CasesStructure,
    // TODO: Move to `CaseSelection` struct
    pub selected_layout: Option<EntryId>,
    pub selected_tab: Option<EntryId>,

    pub tabs: BTreeMap<Path, Layout>,
}

impl CasesState {
    fn autoselect(&mut self) -> Option<()> {
        let mut layout = self.selected_layout.clone().unwrap_or_default();
        let mut tab = self.selected_tab.clone().unwrap_or_default();
        let layouts = &self.structure.layouts;
        if !layouts.contains_key(&layout) {
            layout = layouts.keys().next().cloned()?;
        }
        let tabs = layouts.get(&layout)?;
        if !tabs.contains(&tab) {
            tab = tabs.iter().next().cloned()?;
        }
        self.selected_layout = Some(layout);
        self.selected_tab = Some(tab);
        Some(())
    }
}

// TODO: Implement auto-select

impl CasesState {
    pub fn get_layout_tab(&self) -> Option<&Layout> {
        let selected_layout = self.selected_layout.as_ref()?;
        let selected_tab = self.selected_tab.as_ref()?;
        let path: Path = vec![selected_layout.clone(), selected_tab.clone()].into();
        self.tabs.get(&path)
    }
}

impl Storable for CasesState {
    fn key() -> &'static str {
        module_path!()
    }
}

impl RouterState for CasesState {
    fn restored(&mut self) {
        self.structure.clear();
    }

    fn on_update(&mut self) {
        self.autoselect();
    }
}
