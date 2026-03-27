use crate::check;
use crate::edit;
use crate::insert;
use crate::list;
use crate::remove;
use crate::replace;
use crate::roll;
use crate::search;
use crate::sync;
use crate::utils;

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "nb",
    version,
    about = "Basic notebook CLI that manages a set of rolling notes at different intervals.\nExample: for the daily note, a note is created under notebook/daily/yy-mm-dd.md, on the next day any content that isn't a completed markdown task item \"- [x]\" will be copied to a new note for the current date."
)]
pub struct Cli {
    /// Notebook root directory
    #[arg(long)]
    pub root: Option<PathBuf>,

    /// Editor to use, defaults to $EDITOR
    #[arg(long)]
    pub editor: Option<String>,

    #[command(subcommand)]
    pub cmd: Option<Cmd>,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Edit the current note from a category. Auto-rolls if missing. Defaults to daily.
    Edit(EditArgs),
    /// Appends a line to the selected note type.
    Append(AppendArgs),
    /// Syncs the notebook via git. Make sure your notebook is a git repository with a remote for sync to work.
    Sync,
    /// Generates the notebook structure when notebook root is a nonexistent directory.
    Init,
    /// Searches notebook with rg, make sure you have ripgrep installed.
    Search { query: String },
    /// Prints note content to stdout with line numbers. Takes a note type and option for content or tasks.
    List(ListArgs),
    /// Removes line(s) from the most recent note. Defaults to daily if no note type is provided.
    Remove(RemoveArgs),
    /// Replaces a single line in-place. LINE is 1-based.
    Replace(ReplaceArgs),
    /// Inserts a new line after the given line number. LINE=0 inserts at the top.
    Insert(InsertArgs),
    /// Toggles a markdown task checkbox on the given line.
    Check(CheckArgs),
    /// Manually rolls the note for a given type without opening an editor. Defaults to daily.
    Roll(RollArgs),
}

#[derive(Args, Debug, Default)]
pub struct EditArgs {
    #[arg(value_enum, default_value_t)]
    pub note_type: NoteType,
}

#[derive(Args, Debug, Default)]
pub struct RollArgs {
    #[arg(value_enum, default_value_t)]
    pub note_type: NoteType,
}

#[derive(Args, Debug, Default)]
pub struct ListArgs {
    #[arg(value_enum, default_value_t)]
    pub note_type: NoteType,
    #[arg(value_enum, default_value_t)]
    pub list_type: ListType,
}

#[derive(Args, Debug)]
pub struct AppendArgs {
    #[arg(value_enum)]
    pub note_type: NoteType,

    /// The line to append
    #[arg(value_name = "CONTENT", trailing_var_arg = true)]
    pub content: Vec<String>,
}

#[derive(Args, Debug)]
pub struct ReplaceArgs {
    /// Note type to replace in.
    #[arg(value_enum)]
    pub note_type: NoteType,

    /// Line number to replace (1-based).
    pub line: usize,

    /// The replacement content
    #[arg(value_name = "CONTENT", trailing_var_arg = true)]
    pub content: Vec<String>,
}

#[derive(Args, Debug)]
pub struct InsertArgs {
    /// Note type to insert into.
    #[arg(value_enum)]
    pub note_type: NoteType,

    /// Insert after this line number (0 = insert at top).
    pub line: usize,

    /// The content to insert
    #[arg(value_name = "CONTENT", trailing_var_arg = true)]
    pub content: Vec<String>,
}

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Note type to toggle checkbox in.
    #[arg(value_enum)]
    pub note_type: NoteType,

    /// Line number to toggle (1-based).
    pub line: usize,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// Note type to remove from.
    #[arg(value_enum)]
    pub note_type: NoteType,

    /// Line numbers (1-based). Accepts commas and ranges (e.g. 1,3,5-7).
    #[arg(value_name = "LINES", num_args = 0.., trailing_var_arg = true)]
    pub lines: Vec<String>,
}

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
pub enum NoteType {
    #[default]
    Daily,
    Monthly,
    Yearly,
    Someday,
}

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
pub enum ListType {
    #[default]
    /// Lists all content from the selected note type to stdout
    Contents,
    /// Lists all uncomplete tasks from selected note type to stdout
    Tasks,
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = crate::config::Config::default();
    let cmd = cli.cmd.unwrap_or_else(|| Cmd::Edit(EditArgs::default()));
    match cmd {
        Cmd::Edit(args) => edit::run(args, cfg.clone())?,
        Cmd::Append(args) => edit::append_to_note(args, cfg.clone())?,
        Cmd::Sync => sync::run(cfg.clone())?,
        Cmd::Init => utils::init_notebook(&cfg.root)?,
        Cmd::Search { query } => search::run(cfg.clone(), &query)?,
        Cmd::List(args) => list::run(args, cfg.clone())?,
        Cmd::Remove(args) => remove::run(args, cfg.clone())?,
        Cmd::Replace(args) => replace::run(args, cfg.clone())?,
        Cmd::Insert(args) => insert::run(args, cfg.clone())?,
        Cmd::Check(args) => check::run(args, cfg.clone())?,
        Cmd::Roll(args) => roll::run(args, cfg.clone())?,
    }
    Ok(())
}
