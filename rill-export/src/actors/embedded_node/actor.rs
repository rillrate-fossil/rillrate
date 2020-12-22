use meio::prelude::Actor;

pub struct EmbeddedNode {}

impl Actor for EmbeddedNode {
    type GroupBy = ();
}
