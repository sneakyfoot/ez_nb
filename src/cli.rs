use crate::edit;
use crate::list;
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
    /// Prints note content to stdout. Takes a note type and option for content or tasks.
    List(ListArgs),
}

#[derive(Args, Debug, Default)]
pub struct EditArgs {
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
    }
    Ok(())
}
