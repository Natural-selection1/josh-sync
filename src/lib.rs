pub const DEFAULT_METADATA_DIR: &str = ".josh-sync";
pub const DEFAULT_CONFIG_PATH: &str = ".josh-sync/josh-sync.toml";
pub const DEFAULT_SYNC_VERSION_PATH: &str = ".josh-sync/sync-version";

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
