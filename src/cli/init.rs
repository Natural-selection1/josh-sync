use std::path::Path;

use anyhow::Context as _;

use crate::{
    DEFAULT_BLUEOS_VERSION_PATH, DEFAULT_CONFIG_PATH, config::JoshConfig, sync::FilterVersion,
};

pub fn handle_init() -> Result<(), anyhow::Error> {
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
    match !Path::new(DEFAULT_BLUEOS_VERSION_PATH).is_file() {
        true => {
            std::fs::write(DEFAULT_BLUEOS_VERSION_PATH, "")
                .context("cannot write blueos-version file")?;
            println!("Created empty blueos-version file at {DEFAULT_BLUEOS_VERSION_PATH}");
        }
        false => {
            println!("{DEFAULT_BLUEOS_VERSION_PATH} already exists, not doing anything with it")
        }
    }

    Ok(())
}
