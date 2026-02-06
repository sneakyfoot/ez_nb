use crate::cli::EditArgs;
use crate::config::Config;
use crate::roll;
use crate::utils;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn run(args: EditArgs, cfg: Config) -> anyhow::Result<()> {
    let root = cfg.root.clone();
    let note = utils::resolve_current_note(root.clone(), args.note_type);
    if !note.exists() {
        roll::roll_note(root, args.note_type)?;
    }
    edit_note(note, cfg)?;
    Ok(())
}

fn edit_note(note: PathBuf, cfg: Config) -> anyhow::Result<()> {
    let editor = cfg.editor;
    let status = Command::new(editor)
        .arg(note)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        anyhow::bail!("Editor exited with status: {status}");
    }
    Ok(())
}
