use rate_ui::shared_object::{RouterState, SharedObject};
use rate_ui::storage::typed_storage::Storable;
use rill_protocol::io::provider::{EntryId, Path};
use rrpack_basis::manifest::description::{Layer, PackFlowDescription};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

thread_local! {
    pub static PATHS: SharedObject<DashboardState> = SharedObject::new();
}

pub type Packages = BTreeMap<EntryId, Dashboards>;
pub type Dashboards = BTreeMap<EntryId, Groups>;
pub type Groups = BTreeMap<EntryId, Streams>;
pub type Streams = BTreeSet<EntryId>;

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct DashboardStructure {
    pub packages: Packages,
}

impl DashboardStructure {
    fn clear(&mut self) {
        self.packages.clear();
    }

    pub fn get_packages(&self) -> impl Iterator<Item = &EntryId> {
        self.packages.keys()
    }

    pub fn get_dashboards(&self, package: &EntryId) -> impl Iterator<Item = &EntryId> {
        self.packages
            .get(package)
            .map(BTreeMap::keys)
            .into_iter()
            .flatten()
    }

    pub fn get_groups(&self, selection: &DashboardSelection) -> Option<&Groups> {
        let packages = &self.packages;
        let selected_package = selection.selected_package.as_ref()?;
        let dashboards = packages.get(selected_package)?;
        let selected_dashboard = selection.selected_dashboard.as_ref()?;
        let groups = dashboards.get(selected_dashboard)?;
        Some(groups)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct DashboardSelection {
    pub selected_package: Option<EntryId>,
    pub selected_dashboard: Option<EntryId>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct DashboardState {
    pub structure: DashboardStructure,
    pub selection: DashboardSelection,
    // TODO: Don't copy this part every time!
    pub descs: BTreeMap<Path, PackFlowDescription>,
}

#[derive(Debug, Default)]
pub struct ResolvedDashboard {
    pub visuals: ResolvedLayer,
    pub controls: ResolvedLayer,
    pub transparents: ResolvedLayer,
}

impl DashboardState {
    pub fn autoselect(&mut self) -> Option<()> {
        let mut package = self.selection.selected_package.clone().unwrap_or_default();
        let mut dashboard = self
            .selection
            .selected_dashboard
            .clone()
            .unwrap_or_default();
        let packages = &self.structure.packages;
        if !packages.contains_key(&package) {
            package = packages.keys().next().cloned()?;
        }
        let dashboards = packages.get(&package)?;
        if !dashboards.contains_key(&dashboard) {
            dashboard = dashboards.keys().next().cloned()?;
        }
        self.selection.selected_package = Some(package);
        self.selection.selected_dashboard = Some(dashboard);
        Some(())
    }

    pub fn get_dashboard(&self) -> Option<ResolvedDashboard> {
        let descs = &self.descs;
        let mut resolved_dashboard = ResolvedDashboard::default();
        let groups = self.structure.get_groups(&self.selection)?;
        let selected_package = self.selection.selected_package.as_ref()?;
        let selected_dashboard = self.selection.selected_dashboard.as_ref()?;
        for (group, streams) in groups {
            for stream in streams {
                let path: Path = vec![
                    selected_package.clone(),
                    selected_dashboard.clone(),
                    group.clone(),
                    stream.clone(),
                ]
                .into();
                let desc = descs.get(&path).cloned();
                let layer = desc.as_ref().map(|desc| desc.layer.clone());
                let item = ResolvedItem {
                    name: stream.clone(),
                    //description: desc,
                };
                if let Some(layer) = layer {
                    match layer {
                        Layer::Visual => {
                            resolved_dashboard
                                .visuals
                                .entry(group.clone())
                                .or_default()
                                .insert(path, item);
                        }
                        Layer::Control => {
                            resolved_dashboard
                                .controls
                                .entry(group.clone())
                                .or_default()
                                .insert(path, item);
                        }
                        Layer::Transparent => {
                            resolved_dashboard
                                .transparents
                                .entry(group.clone())
                                .or_default()
                                .insert(path, item);
                        }
                    }
                }
            }
        }

        /*
        let mut visuals = BTreeMap::new();
        let mut controls = BTreeMap::new();
        let selected_package = self.selection.selected_package.as_ref()?;
        let selected_dashboard = self.selection.selected_dashboard.as_ref()?;

        let flows = self
            .structure
            .get_streams(&self.selection)
            .unwrap_or_default();
        for flow in flows {
            let path = vec![
                selected_package.clone(),
                selected_dashboard.clone(),
                flow.clone(),
            ]
            .into();
            let desc = descs.get(&path).cloned();
            let layer = desc.as_ref().map(|desc| desc.layer.clone());
            let item = ResolvedItem {
                name: flow,
                description: desc,
            };
            if let Some(layer) = layer {
                match layer {
                    Layer::Visual => {
                        visuals.insert(path, item);
                    }
                    Layer::Control => {
                        controls.insert(path, item);
                    }
                    Layer::Transparent => {}
                }
            }
        }
        Some((visuals, controls))
        */
        Some(resolved_dashboard)
    }
}

pub type ResolvedLayer = BTreeMap<EntryId, ResolvedGroup>;

// TODO: Change to BTreeSet
pub type ResolvedGroup = BTreeMap<Path, ResolvedItem>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedItem {
    pub name: EntryId,
    //pub description: Option<PackFlowDescription>,
}

impl Storable for DashboardState {
    fn key() -> &'static str {
        module_path!()
    }
}

impl RouterState for DashboardState {
    fn restored(&mut self) {
        self.structure.clear();
    }

    fn on_update(&mut self) {
        self.autoselect();
    }
}
