use super::Exporter;
use derive_more::From;
use meio::prelude::Address;

#[derive(Debug, Clone, From)]
pub struct ExporterLink {
    address: Address<Exporter>,
}
