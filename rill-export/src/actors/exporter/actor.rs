use meio::prelude::Actor;

/// The `Actor` that subscribes to data according to available `Path`s.
pub struct Exporter {}

impl Actor for Exporter {
    type GroupBy = ();
}
