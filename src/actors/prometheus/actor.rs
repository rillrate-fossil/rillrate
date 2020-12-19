use meio::prelude::Actor;

pub struct PrometheusExporter {}

impl Actor for PrometheusExporter {
    type GroupBy = ();
}
