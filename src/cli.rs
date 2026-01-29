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
    pub note_type: NoteType,
}

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
pub enum NoteType {
    #[default]
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Someday,
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cmd = cli.cmd.unwrap_or_else(|| Cmd::Edit(EditArgs::default()));
    match cmd {
        Cmd::Edit(args) => match args.note_type {
            NoteType::Daily => println!("edit daily"),
            NoteType::Weekly => println!("edit weekly"),
            NoteType::Monthly => println!("edit monthly"),
            NoteType::Yearly => println!("edit yearly"),
            NoteType::Someday => println!("edit someday"),
        },
    }
    Ok(())
}
