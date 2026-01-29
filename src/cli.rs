use crate::edit;

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "nb", version, about = "Notebook CLI")]
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
    Edit(EditArgs),
}

#[derive(Args, Debug, Default)]
pub struct EditArgs {
    #[arg(value_enum, default_value_t)]
    pub note_type: NoteType,
}

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
pub enum NoteType {
    #[default]
    Daily,
    Monthly,
    Yearly,
    Someday,
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = crate::config::Config::default();
    let cmd = cli.cmd.unwrap_or_else(|| Cmd::Edit(EditArgs::default()));
    match cmd {
        Cmd::Edit(args) => edit::run(args, cfg.clone())?,
    }
    Ok(())
}
