use std::path::Path;

use anyhow::Context as _;

use crate::{
    DEFAULT_CONFIG_PATH, DEFAULT_METADATA_DIR, DEFAULT_SYNC_VERSION_PATH, config::JoshConfig,
    sync::FilterVersion,
};

pub fn handle_init() -> Result<(), anyhow::Error> {
    std::fs::create_dir_all(DEFAULT_METADATA_DIR).context("cannot create .josh-sync directory")?;

    let config = JoshConfig {
        org: "vivoblueos".to_string(),
        repo: "<repository-name>".to_string(),
        upstream_repo: "<github-owner>/<blueos-monorepo>".to_string(),
        upstream_branch: "main".to_string(),
        path: Some("<relative-subtree-path>".to_string()),
        filter: None,
        post_pull: vec![],
        subtree_filter: None,
        filter_version: FilterVersion::latest(),
    };
    config
        .write(Path::new(DEFAULT_CONFIG_PATH))
        .context("cannot write config")?;
    println!("Created config file at {DEFAULT_CONFIG_PATH}");
    match !Path::new(DEFAULT_SYNC_VERSION_PATH).is_file() {
        true => {
            std::fs::write(DEFAULT_SYNC_VERSION_PATH, "")
                .context("cannot write sync-version file")?;
            println!("Created empty sync-version file at {DEFAULT_SYNC_VERSION_PATH}");
        }
        false => {
            println!("{DEFAULT_SYNC_VERSION_PATH} already exists, not doing anything with it")
        }
    }

    Ok(())
}
