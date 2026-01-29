use crate::cli::NoteType;
use anyhow::Context;
use chrono::{Datelike, Local};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub fn read_note(path: &Path) -> anyhow::Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

pub fn write_note(path: &Path, contents: &str) -> anyhow::Result<()> {
    std::fs::write(path, contents)?;
    Ok(())
}

pub fn construct_header(note_type: NoteType) -> String {
    let today = Local::now().date_naive();
    let header = match note_type {
        NoteType::Daily => format!(
            "## {} {:02}-{:02}-{:02}",
            today.format("%A"),
            today.year() % 100,
            today.month(),
            today.day()
        ),
        NoteType::Monthly => format!("## {} {:04}", today.format("%B").to_string(), today.year(),),
        NoteType::Yearly => format!("## {:04}", today.year()),
        NoteType::Someday => format!("## Someday"),
    };
    header
}

/// Figures out what the current note for each catagory would be on today.
/// This can return a filename that does not exist yet
pub fn resolve_current_note(root: PathBuf, note_type: NoteType) -> PathBuf {
    let today = Local::now().date_naive();
    match note_type {
        NoteType::Daily => {
            let folder = root.join("daily");
            let filename = format!(
                "{:02}-{:02}-{:02}.md",
                today.year() % 100,
                today.month(),
                today.day()
            );
            folder.join(filename)
        }
        NoteType::Monthly => {
            let folder = root.join("monthly");
            let filename = format!("{:02}-{:02}.md", today.year() % 100, today.month());
            folder.join(filename)
        }
        NoteType::Yearly => {
            let folder = root.join("yearly");
            let filename = format!("{:04}.md", today.year());
            folder.join(filename)
        }
        NoteType::Someday => root.join("someday.md"),
    }
}

/// Find the most recent note that does exist, based on NoteType.
/// Sort assumes YY MM DD
pub fn resolve_most_recent_note(
    root: PathBuf,
    note_type: NoteType,
) -> anyhow::Result<Option<PathBuf>> {
    let folder = match note_type {
        NoteType::Daily => root.join("daily"),
        NoteType::Monthly => root.join("monthly"),
        NoteType::Yearly => root.join("yearly"),
        NoteType::Someday => root,
    };
    let latest = std::fs::read_dir(&folder)
        .with_context(|| format!("read_dir failed: {}", folder.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| p.extension() == Some(OsStr::new("md")))
        .filter_map(|p| {
            let stem = p.file_stem()?.to_str()?;
            Some((stem.to_string(), p))
        })
        .max_by_key(|(stem, _path)| stem.clone())
        .map(|(_stem, path)| path);
    Ok(latest)
}
