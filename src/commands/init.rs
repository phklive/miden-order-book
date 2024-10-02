use clap::Parser;
use std::{fs, path::Path};

#[derive(Debug, Clone, Parser)]
#[clap(about = "Setup the order book")]
pub struct InitCmd {}

impl InitCmd {
    pub fn execute(&self) -> Result<(), String> {
        let file_path = Path::new("store.sqlite3");
        if file_path.exists() {
            println!("Deleting store.sqlite3");
            fs::remove_file(file_path).map_err(|e| format!("Failed to remove file: {}", e))?;
            println!("File deleted successfully");
        } else {
            println!("store.sqlite3 does not exist");
        }
        Ok(())
    }
}
