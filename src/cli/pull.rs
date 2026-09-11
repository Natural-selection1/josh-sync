use crate::{
    cli::parser::SharedArgs,
    josh::get_josh_proxy,
    sync::{BlueosPullError, GitSync, load_context},
    utils::maybe_create_gh_pr,
};

pub fn handle_pull(
    upstream_repo: Option<String>,
    upstream_branch: Option<String>,
    upstream_commit: Option<String>,
    allow_noop: bool,
    shared: SharedArgs,
) -> Result<(), anyhow::Error> {
    let ctx = load_context(&shared.config_path, &shared.sync_version_path)?;
    let josh = get_josh_proxy(shared.josh_proxy, shared.verbose)?;
    let sync = GitSync::new(ctx.clone(), josh, shared.verbose);
    let upstream_repo = upstream_repo.unwrap_or_else(|| ctx.config.upstream_repo.clone());
    let upstream_branch = upstream_branch.unwrap_or_else(|| ctx.config.upstream_branch.clone());

    match sync.blueos_pull(upstream_repo, upstream_branch, upstream_commit, allow_noop) {
        Ok(result) => {
            if !maybe_create_gh_pr(
                &ctx.config.full_repo_name(),
                "BlueOS pull update",
                &result.merge_commit_message,
            )? {
                println!(
                    "Now push the current branch to {} (either a fork or the main repo) and create a PR",
                    ctx.config.repo
                );
            }
        }
        Err(BlueosPullError::NothingToPull) => {
            eprintln!("Nothing to pull");
            if !allow_noop {
                std::process::exit(2);
            }
        }
        Err(BlueosPullError::PullFailed(error)) => {
            eprintln!("Pull failure: {error:?}");
            if !shared.verbose {
                eprintln!("Rerun with `-v` to see executed commands");
            }
            std::process::exit(1);
        }
    }
    Ok(())
}
