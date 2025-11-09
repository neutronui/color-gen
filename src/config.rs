use std::path::PathBuf;
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
  pub transforms: Vec<Transform>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
  pub from: PathBuf,
  pub to: Vec<TransformTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformTarget {
  pub format: TargetFormat,
  pub output: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetFormat {
  Json,
  Toml,
  Yaml,
  Scss,
  Css,
  Mjs
}

pub mod cli {
  use clap::Parser;
  use open::that;
  use simply_colored::*;
  use crate::APP_DIRS;

  #[derive(Debug, Parser)]
  pub enum Commands {
    Path,
    Edit
  }

  pub fn handle(cmd: &Commands) {
    match cmd {
      Commands::Path => {
        println!("{DIM_YELLOW}Config path: {RESET}{BOLD}{:?}{RESET}", APP_DIRS.config_dir);
      },
      Commands::Edit => {
        let config_path = APP_DIRS.config_dir.join("config.toml");
        if let Err(e) = that(config_path) {
          eprintln!("{RED}Failed to open config file: {RESET}{e}");
        }
      }
    }
  }
}
