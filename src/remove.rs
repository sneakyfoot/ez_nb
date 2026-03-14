use crate::cli::{NoteType, RemoveArgs};
use crate::config::Config;
use crate::utils;
use anyhow::{Context, bail};
use std::collections::HashSet;

pub fn run(args: RemoveArgs, cfg: Config) -> anyhow::Result<()> {
    let note_type = args.note_type;
    if args.lines.is_empty() {
        return Ok(());
    }
    let line_numbers = parse_line_numbers(&args.lines)?;
    let root = cfg.root.clone();
    let note = utils::resolve_most_recent_note(root, note_type)?
        .ok_or_else(|| anyhow::anyhow!("No note found for {}", note_type_label(note_type)))?;
    let content = utils::read_note(&note)?;
    let has_trailing_newline = content.ends_with('\n');
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        bail!("Note is empty, nothing to remove.");
    }

    let mut remove_set: HashSet<usize> = HashSet::new();
    for num in line_numbers {
        if num == 0 {
            bail!("Line numbers are 1-based.");
        }
        if num > lines.len() {
            bail!("Line {} is out of range (1-{}).", num, lines.len());
        }
        remove_set.insert(num);
    }

    let mut kept: Vec<&str> = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let line_no = idx + 1;
        if remove_set.contains(&line_no) {
            continue;
        }
        kept.push(*line);
    }

    let mut output = kept.join("\n");
    if has_trailing_newline && !output.is_empty() {
        output.push('\n');
    }
    utils::write_note(&note, &output)?;
    Ok(())
}

fn parse_line_numbers(args: &[String]) -> anyhow::Result<Vec<usize>> {
    if args.is_empty() {
        bail!("No line numbers provided.");
    }
    let mut nums = Vec::new();
    for arg in args {
        for raw in arg.split(',') {
            let part = raw.trim();
            if part.is_empty() {
                continue;
            }
            let dash_count = part.matches('-').count();
            if dash_count == 0 {
                nums.push(parse_line_number(part)?);
                continue;
            }
            if dash_count > 1 {
                bail!("Invalid range: {}", part);
            }
            let (start_s, end_s) = part.split_once('-').unwrap();
            let start = parse_line_number(start_s.trim())?;
            let end = parse_line_number(end_s.trim())?;
            if start > end {
                bail!("Invalid range (start > end): {}", part);
            }
            for n in start..=end {
                nums.push(n);
            }
        }
    }
    if nums.is_empty() {
        bail!("No line numbers provided.");
    }
    Ok(nums)
}

fn parse_line_number(value: &str) -> anyhow::Result<usize> {
    if value.is_empty() {
        bail!("Invalid line number: empty");
    }
    let num: usize = value
        .parse()
        .with_context(|| format!("Invalid line number: {}", value))?;
    Ok(num)
}

fn note_type_label(note_type: NoteType) -> &'static str {
    match note_type {
        NoteType::Daily => "daily",
        NoteType::Monthly => "monthly",
        NoteType::Yearly => "yearly",
        NoteType::Someday => "someday",
    }
}
