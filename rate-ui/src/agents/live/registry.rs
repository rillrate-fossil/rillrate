use once_cell::sync::Lazy;
use rill_protocol::io::client::ClientReqId;
use std::sync::Mutex;
use typed_slab::TypedSlab;

struct RegistryInner {
    ids: TypedSlab<ClientReqId, ()>,
}

pub struct Registry {
    inner: Mutex<RegistryInner>,
}

impl Registry {
    fn new() -> Self {
        let inner = RegistryInner {
            ids: TypedSlab::new(),
        };
        Self {
            inner: Mutex::new(inner),
        }
    }
}

pub static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

impl Registry {
    pub fn acquire(&self) -> ClientReqId {
        self.inner.lock().unwrap().ids.insert(())
    }

    pub fn release(&self, req_id: ClientReqId) {
        self.inner.lock().unwrap().ids.remove(req_id);
    }
}
