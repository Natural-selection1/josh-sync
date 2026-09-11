use clap::Parser as _;
use josh_sync::cli::init::handle_init;
use josh_sync::cli::parser::{Args, Command};
use josh_sync::cli::pull::handle_pull;
use josh_sync::cli::push::handle_push;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.cmd {
        Command::Init => handle_init(),
        Command::Pull {
            upstream_repo,
            upstream_branch,
            upstream_commit,
            allow_noop,
            shared,
        } => handle_pull(
            upstream_repo,
            upstream_branch,
            upstream_commit,
            allow_noop,
            shared,
        ),
        Command::Push {
            branch,
            username,
            shared,
        } => handle_push(branch, username, shared),
    }
}
