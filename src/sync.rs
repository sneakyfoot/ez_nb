use crate::config::Config;
use anyhow::{Context, anyhow};
use chrono::{Datelike, Local};
use std::path::Path;
use std::process::Command;

pub fn run(cfg: Config) -> anyhow::Result<()> {
    let root = cfg.root;
    if let Err(e) = git_pull(&root) {
        eprintln!("Git pull failed, is your notebook tracked by git?");
        eprintln!("See `nb sync --help` for more information");
        eprintln!("Details: {e:#}");
        return Ok(());
    }
    let changes = check_git_status(&root).context("Git status failed")?;
    if changes.trim().is_empty() {
        println!("No changes, skipping sync");
        return Ok(());
    }
    println!("Changed files:\n{changes}");
    git_add_all(&root)?;
    git_commit(&root)?;
    git_push(&root)?;
    Ok(())
}

fn check_git_status(root: &Path) -> anyhow::Result<String> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["status", "--porcelain"])
        .output()
        .context("Git status failed")?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("Git status failed: {err}");
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn git_add_all(root: &Path) -> anyhow::Result<()> {
    git(root, ["add", "--all"]).context("Git add failed")?;
    Ok(())
}

fn git_commit(root: &Path) -> anyhow::Result<()> {
    let message = build_commit_message();
    let res = git(root, ["commit", "-m", &message]);
    match res {
        Ok(()) => Ok(()),
        Err(e) => {
            // Treat "nothing to commit" as non-fatal.
            if format!("{e:#}").contains("nothing to commit") {
                println!("Nothing to commit");
                Ok(())
            } else {
                Err(e).context("Git commit failed")
            }
        }
    }
}

fn git_push(root: &Path) -> anyhow::Result<()> {
    git(root, ["push"]).context("Git push failed")?;
    Ok(())
}

fn git_pull(root: &Path) -> anyhow::Result<()> {
    git(root, ["pull", "--rebase", "--autostash"]).context("Git pull failed")?;
    Ok(())
}

fn git<const N: usize>(root: &Path, args: [&str; N]) -> anyhow::Result<()> {
    let out = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run git {}", args.join(" ")))?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(anyhow!("git {} failed: {err}", args.join(" ")));
    }
    Ok(())
}

fn build_commit_message() -> String {
    let today = Local::now().date_naive();
    format!(
        "Sync {:02}-{:02}-{:02}",
        today.year() % 100,
        today.month(),
        today.day()
    )
}
