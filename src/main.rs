mod cli;
mod config;
mod edit;
mod roll;
mod sync;
mod utils;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
