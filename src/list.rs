use crate::cli::ListArgs;
use crate::cli::ListType;
use crate::config::Config;
use crate::utils;
use std::fs;

pub fn run(args: ListArgs, cfg: Config) -> anyhow::Result<()> {
    let root = cfg.root.clone();
    let note = utils::resolve_most_recent_note(root.clone(), args.note_type)?.unwrap();
    let content = fs::read_to_string(note)?;
    let output = match args.list_type {
        ListType::Contents => content,
        ListType::Tasks => keep_open_tasks(&content),
    };
    println!("{output}");
    Ok(())
}

fn keep_open_tasks(text: &str) -> String {
    text.lines()
        .filter(|line| line.contains("- [ ]"))
        .collect::<Vec<_>>()
        .join("\n")
}
