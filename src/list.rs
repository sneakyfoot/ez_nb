use crate::cli::ListArgs;
use crate::cli::ListType;
use crate::config::Config;
use crate::utils;
use std::fs;

pub fn run(args: ListArgs, cfg: Config) -> anyhow::Result<()> {
    let root = cfg.root.clone();
    let note = utils::resolve_most_recent_note(root.clone(), args.note_type)?.unwrap();
    let content = fs::read_to_string(note)?;
    let lines: Vec<&str> = content.lines().collect();
    let width = lines.len().to_string().len().max(1);
    let output = match args.list_type {
        ListType::Contents => format_lines(&lines, width),
        ListType::Tasks => format_open_tasks(&lines, width),
    };
    println!("{output}");
    Ok(())
}

fn format_lines(lines: &[&str], width: usize) -> String {
    lines
        .iter()
        .enumerate()
        .map(|(idx, line)| format!("{:>width$} | {}", idx + 1, line, width = width))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_open_tasks(lines: &[&str], width: usize) -> String {
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains("- [ ]"))
        .map(|(idx, line)| format!("{:>width$} | {}", idx + 1, line, width = width))
        .collect::<Vec<_>>()
        .join("\n")
}
