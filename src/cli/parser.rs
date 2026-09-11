use std::path::PathBuf;

use crate::{DEFAULT_CONFIG_PATH, DEFAULT_SYNC_VERSION_PATH};

#[derive(clap::Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(clap::Parser)]
pub enum Command {
    /// Initialize `.josh-sync/josh-sync.toml` and `.josh-sync/sync-version`.
    Init,
    /// Pull changes from the BlueOS monorepo configured in `.josh-sync/josh-sync.toml`.
    /// This creates new commits that should be then merged into this subtree repository.
    Pull {
        /// Override the configured upstream repository for a local experiment.
        /// CI should use the checked-in `upstream-repo` instead.
        #[clap(long)]
        upstream_repo: Option<String>,

        /// Override the configured upstream branch for a local experiment.
        /// CI should use the checked-in `upstream-branch` instead.
        #[clap(long)]
        upstream_branch: Option<String>,

        /// Override the BlueOS monorepo commit that we should pull from.
        /// By default, josh-sync resolves the configured BlueOS monorepo branch.
        #[clap(long)]
        upstream_commit: Option<String>,

        /// By default, the `pull` command will exit with status code 2 if there is nothing to pull,
        /// and reset git to the original state.
        /// If you instead want to exit successfully and keep the intermediate changes
        /// in that case, pass this flag.
        #[clap(long)]
        allow_noop: bool,
        #[clap(flatten)]
        shared: SharedArgs,
    },
    /// Push changes into `branch` of a fork of the configured BlueOS monorepo under the given
    /// GitHub `username`.
    /// The pushed branch should then be merged into the BlueOS monorepo.
    Push {
        /// Branch that should be pushed to your remote
        branch: String,

        /// Your GitHub usename where the fork is located
        username: String,
        #[clap(flatten)]
        shared: SharedArgs,
    },
}

#[derive(clap::Parser)]
pub struct SharedArgs {
    /// Path to the josh-sync TOML config file.
    #[clap(long, default_value(DEFAULT_CONFIG_PATH))]
    pub config_path: PathBuf,

    /// Path to a file storing the last synchronized BlueOS monorepo commit.
    #[clap(long, default_value(DEFAULT_SYNC_VERSION_PATH))]
    pub sync_version_path: PathBuf,

    /// Path to the josh-proxy binary to be used.
    /// If not specified, it will be installed automatically.
    ///
    /// Warning: if you use a custom Josh version, ensure that it works properly!
    #[clap(long)]
    pub josh_proxy: Option<PathBuf>,

    /// Print executed commands.
    #[clap(long, short = 'v', env = "JOSH_SYNC_VERBOSE")]
    pub verbose: bool,
}
