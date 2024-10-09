use clap::Parser;
use std::{fs, path::Path};

use crate::constants::{DB_FILE_PATH, DETAILS_FILE_PATH, USER_ACCOUNT_FILE_PATH};

#[derive(Debug, Clone, Parser)]
#[clap(about = "Initialize the order book")]
pub struct InitCmd {}

impl InitCmd {
    pub fn execute(&self) -> Result<(), String> {
        Self::remove_file_if_exists(DB_FILE_PATH)?;
        Self::remove_file_if_exists(DETAILS_FILE_PATH)?;
        Self::remove_file_if_exists(USER_ACCOUNT_FILE_PATH)?;
        Ok(())
    }

    fn remove_file_if_exists(file_path: &str) -> Result<(), String> {
        let path = Path::new(file_path);
        if path.exists() {
            println!("Deleting {}", file_path);
            fs::remove_file(path)
                .map_err(|e| format!("Failed to remove file {}: {}", file_path, e))?;
            println!("File deleted successfully");
        } else {
            println!("{} does not exist", file_path);
        }
        Ok(())
    }
}
