use crate::cli::NoteType;
use crate::utils;
use std::path::PathBuf;

pub fn roll_note(root: PathBuf, note_type: NoteType) -> anyhow::Result<()> {
    let current_note = utils::resolve_current_note(root.clone(), note_type);
    if current_note.try_exists()? {
        anyhow::bail!(
            "Trying to roll into a file that already exists: {}, bailing",
            current_note.display()
        );
    }
    let latest_note = utils::resolve_most_recent_note(root.clone(), note_type)?
        .ok_or_else(|| anyhow::anyhow!("Could not resolve latest note"))?;
    let header = utils::construct_header(note_type);
    let current_content = utils::read_note(&latest_note)?;
    let new_content = construct_new_note(&current_content, &header);
    utils::write_note(&current_note, &new_content)?;
    Ok(())
}

fn construct_new_note(prev: &str, header: &str) -> String {
    let mut out = String::new();
    out.push_str(header.trim_end());
    out.push('\n');
    let mut skipped_old_header = false;
    for line in prev.lines() {
        if !skipped_old_header {
            if line.trim().is_empty() {
                continue;
            } else {
                skipped_old_header = true;
                continue;
            }
        }
        let is_done = line.trim_start().starts_with("- [x]");
        if is_done {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
