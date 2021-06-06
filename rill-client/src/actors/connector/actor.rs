use meio::Actor;

pub struct Connector {
    url: String,
}

impl Connector {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Actor for Connector {
    type GroupBy = ();
}
