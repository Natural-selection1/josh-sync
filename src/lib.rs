pub const DEFAULT_CONFIG_PATH: &str = "josh-sync.toml";
pub const DEFAULT_BLUEOS_VERSION_PATH: &str = "blueos-version";

pub mod cli {
    pub mod init;
    pub mod parser;
    pub mod pull;
    pub mod push;
}
pub mod config;
pub mod josh;
pub mod sync;
pub mod utils;
