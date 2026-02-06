use crate::config::Config;
use anyhow::bail;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use which::which;

pub fn run(config: Config, query: &str) -> anyhow::Result<()> {
    check_rg()?;
    let root = config.root;
    let results = search(root, &query)?;
    let output = dedupe(&results);
    println!("{}", output);
    Ok(())
}

fn search(root: PathBuf, query: &str) -> anyhow::Result<String> {
    let out = Command::new("rg")
        .current_dir(root)
        .args(["-F", query])
        .output()?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        bail!("rg failed: {err}");
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn dedupe(results: &str) -> String {
    let mut best: HashMap<String, (String, String)> = HashMap::new();
    for line in results.lines() {
        let Some((path, rest)) = line.split_once(':') else {
            continue;
        };
        let Some(date_key) = date_key_from_path(path) else {
            continue;
        };
        let task = normalize_task(rest);
        match best.get(&task) {
            Some((prev_key, _)) if prev_key >= &date_key => {}
            _ => {
                best.insert(task, (date_key, line.to_string()));
            }
        }
    }
    let mut vals: Vec<(String, String)> = best.into_values().collect();
    vals.sort_by(|a, b| b.0.cmp(&a.0));
    vals.into_iter()
        .map(|(_, l)| l)
        .collect::<Vec<_>>()
        .join("\n")
}

fn date_key_from_path(path: &str) -> Option<String> {
    let fname = path.rsplit('/').next()?;
    let stem = fname.strip_suffix(".md")?;
    Some(stem.to_string())
}

fn normalize_task(text: &str) -> String {
    let content = text.trim();
    content
        .strip_prefix("- [ ] ")
        .or_else(|| content.strip_prefix("- [x] "))
        .unwrap_or(content)
        .trim()
        .to_string()
}

fn check_rg() -> anyhow::Result<()> {
    if which("rg").is_err() {
        bail!("rg is not found, Install ripgrep to use search");
    }
    Ok(())
}
