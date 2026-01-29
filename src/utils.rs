use crate::cli::NoteType;
use anyhow::Context;
use chrono::{Datelike, Local};
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn read_note(path: &Path) -> anyhow::Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

pub fn write_note(path: &Path, contents: &str) -> anyhow::Result<()> {
    std::fs::write(path, contents)?;
    Ok(())
}

pub fn confirm(prompt: &str) -> io::Result<bool> {
    loop {
        eprintln!("{prompt} [y/n]: ");
        io::stderr().flush()?;

        let mut s = String::new();
        io::stdin().read_line(&mut s)?;

        match s.trim().to_ascii_lowercase().as_str() {
            "y" => return Ok(true),
            "n" => return Ok(false),
            _ => eprintln!("Type y or n"),
        }
    }
}

pub fn init_notebook(root: &Path) -> anyhow::Result<()> {
    let conrimation_message = format!(
        "This will create a new notebook at {}, confirm?",
        root.display()
    );
    if confirm(&conrimation_message)? {
        if root.exists() {
            anyhow::bail!("Notebook root already exists, bailing");
        }
        fs::create_dir_all(root)
            .with_context(|| format!("Failed to create notebook root: {}", root.display()))?;
        for name in ["daily", "monthly", "yearly"] {
            let dir = root.join(name);
            fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create folder: {}", dir.display()))?;
            let placeholder = dir.join("placeholder");
            fs::write(&placeholder, "placeholder".to_string()).with_context(|| {
                format!("Failed to create placeholder: {}", placeholder.display())
            })?;
        }
        let someday_note = root.join("someday.md");
        fs::write(&someday_note, "## Someday")
            .with_context(|| format!("Failed to write someday.md"))?;
    }
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
