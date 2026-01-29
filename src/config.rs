use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub root: PathBuf,
    pub editor: String,
}

impl Default for Config {
    fn default() -> Self {
        let home = PathBuf::from(std::env::var("HOME").expect("$HOME not set"));
        let root = home.join("notebook");
        let editor = std::env::var("EDITOR").unwrap_or("nvim".to_string());
        Self { root, editor }
    }
}
