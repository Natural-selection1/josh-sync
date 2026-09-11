use anyhow::Context as _;

use crate::{
    cli::parser::SharedArgs,
    josh::get_josh_proxy,
    sync::{GitSync, load_context},
    utils::get_current_head_sha,
};

pub fn handle_push(
    branch: String,
    username: String,
    shared: SharedArgs,
) -> Result<(), anyhow::Error> {
    let ctx = load_context(&shared.config_path, &shared.blueos_version_path)?;
    let josh = get_josh_proxy(shared.josh_proxy, shared.verbose)?;
    let sync = GitSync::new(ctx.clone(), josh, shared.verbose);
    if let Err(error) = sync
        .blueos_push(&username, &branch)
        .context("cannot perform push")
    {
        if !shared.verbose {
            eprintln!("Rerun with `-v` to see executed commands");
        }
        return Err(error);
    }
    let title = format!("{} subtree update", ctx.config.repo);
    let head = get_current_head_sha(shared.verbose)?;
    let merge_msg = format!(
        r#"Subtree update of `{repo}` to https://github.com/{full_repo}/commit/{head}.

Created using vivoblueos-josh-sync.

r? @ghost"#,
        repo = ctx.config.repo,
        full_repo = ctx.config.full_repo_name(),
    );
    println!(
        r#"You can create the BlueOS monorepo PR using the following URL:
https://github.com/{upstream_repo}/compare/{upstream_branch}...{username}:{branch}?quick_pull=1&title={}&body={}"#,
        urlencoding::encode(&title),
        urlencoding::encode(&merge_msg),
        upstream_repo = ctx.config.upstream_repo,
        upstream_branch = ctx.config.upstream_branch
    );
    Ok(())
}
