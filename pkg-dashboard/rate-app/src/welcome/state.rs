use rate_ui::shared_object::{RouterState, SharedObject};
use rate_ui::storage::typed_storage::Storable;
use serde::{Deserialize, Serialize};
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
    pub global_scene: GlobalScene,
}

impl Storable for SceneState {
    fn key() -> &'static str {
        module_path!()
    }
}

impl RouterState for SceneState {
    fn restored(&mut self) {
        self.global_scene = Default::default();
    }
}
