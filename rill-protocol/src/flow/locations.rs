pub mod server {
    use crate::io::provider::Path;

    pub fn alerts() -> Path {
        "@server.alerts".parse().unwrap()
    }
}
