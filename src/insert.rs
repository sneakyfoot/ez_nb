use crate::cli::{InsertArgs, NoteType};
use crate::config::Config;
use crate::utils;
use anyhow::bail;

pub fn run(args: InsertArgs, cfg: Config) -> anyhow::Result<()> {
    let note_type = args.note_type;
    let root = cfg.root.clone();
    let note = utils::resolve_most_recent_note(root, note_type)?
        .ok_or_else(|| anyhow::anyhow!("No note found for {}", note_type_label(note_type)))?;
    let content = utils::read_note(&note)?;
    let has_trailing_newline = content.ends_with('\n');
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let line = args.line;
    if line > lines.len() {
        bail!("Line {} is out of range (0-{}).", line, lines.len());
    }
    let new_line = args.content.join(" ");
    lines.insert(line, new_line);
    let mut output = lines.join("\n");
    if has_trailing_newline {
        output.push('\n');
    }
    utils::write_note(&note, &output)?;
    Ok(())
}

fn note_type_label(note_type: NoteType) -> &'static str {
    match note_type {
        NoteType::Daily => "daily",
        NoteType::Monthly => "monthly",
        NoteType::Yearly => "yearly",
        NoteType::Someday => "someday",
    }
}
