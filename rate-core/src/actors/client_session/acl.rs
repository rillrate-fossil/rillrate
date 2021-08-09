use rill_protocol::flow::location::Location;
use rill_protocol::io::provider::{EntryId, Path};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SessionAcl {
    session_id: EntryId,
    inner: Arc<Mutex<SessionAclInner>>,
}

impl SessionAcl {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        // @ prefix means - hidden
        let session_id = format!("@{}", Uuid::new_v4()).into();
        let inner = SessionAclInner {
            unlock_all: false,
            allowed_paths: HashSet::new(),
        };
        Self {
            session_id,
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub fn relative_path(&self, location: &Location) -> Path {
        location.of(self.session_id.clone().into())
    }

    pub fn id(&self) -> &EntryId {
        &self.session_id
    }

    pub async fn add_path(&mut self, path: Path) {
        self.inner.lock().await.allowed_paths.insert(path);
    }

    pub async fn remove_path(&mut self, path: &Path) {
        self.inner.lock().await.allowed_paths.remove(path);
    }

    pub async fn unlock_all(&mut self) {
        self.inner.lock().await.unlock_all = true;
    }

    pub async fn lock_all(&mut self) {
        self.inner.lock().await.unlock_all = false;
    }

    pub async fn has_access_to(&mut self, path: &Path) -> bool {
        let inner = self.inner.lock().await;
        inner.unlock_all || inner.allowed_paths.contains(path)
    }
}

#[derive(Debug)]
struct SessionAclInner {
    // TODO: Improve. It's a temporary solution for the `Node` app only.
    unlock_all: bool,
    allowed_paths: HashSet<Path>,
}
