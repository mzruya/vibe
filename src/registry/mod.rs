pub mod formula;
pub mod github;
pub mod local;

pub use github::{GitHubRegistry, parse_package_spec};
pub use local::LocalRegistry;
