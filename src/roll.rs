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
    println!(
        "this would roll {} into {}\n with the header: {}",
        latest_note.display(),
        current_note.display(),
        header
    );
    Ok(())
}
