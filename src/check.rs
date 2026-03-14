use crate::cli::{CheckArgs, NoteType};
use crate::config::Config;
use crate::utils;
use anyhow::bail;

pub fn run(args: CheckArgs, cfg: Config) -> anyhow::Result<()> {
    let note_type = args.note_type;
    let root = cfg.root.clone();
    let note = utils::resolve_most_recent_note(root, note_type)?
        .ok_or_else(|| anyhow::anyhow!("No note found for {}", note_type_label(note_type)))?;
    let content = utils::read_note(&note)?;
    let has_trailing_newline = content.ends_with('\n');
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        bail!("Note is empty, nothing to check.");
    }
    let line = args.line;
    if line == 0 {
        bail!("Line numbers are 1-based.");
    }
    if line > lines.len() {
        bail!("Line {} is out of range (1-{}).", line, lines.len());
    }
    let original = lines[line - 1];
    let toggled = toggle_checkbox(original)?;
    let new_lines: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(idx, &l)| {
            if idx + 1 == line {
                toggled.clone()
            } else {
                l.to_string()
            }
        })
        .collect();
    let mut output = new_lines.join("\n");
    if has_trailing_newline {
        output.push('\n');
    }
    utils::write_note(&note, &output)?;
    Ok(())
}

fn toggle_checkbox(line: &str) -> anyhow::Result<String> {
    let trimmed = line.trim_start();
    let leading_ws = &line[..line.len() - trimmed.len()];
    if trimmed.starts_with("- [ ]") {
        Ok(format!("{}- [x]{}", leading_ws, &trimmed[5..]))
    } else if trimmed.starts_with("- [x]") {
        Ok(format!("{}- [ ]{}", leading_ws, &trimmed[5..]))
    } else {
        bail!("Line does not contain a markdown checkbox (\"- [ ]\" or \"- [x]\").");
    }
}

fn note_type_label(note_type: NoteType) -> &'static str {
    match note_type {
        NoteType::Daily => "daily",
        NoteType::Monthly => "monthly",
        NoteType::Yearly => "yearly",
        NoteType::Someday => "someday",
    }
}
