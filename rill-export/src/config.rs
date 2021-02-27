use rill_protocol::config::ConfigPatch;

// TODO: Support no env vars for `ConfigPatch`
pub static NODE: ConfigPatch<String> = ConfigPatch::new("VAR-NOT-SPECIFIED");
