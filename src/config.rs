use std::path::PathBuf;
use serde::{Serialize, Deserialize};

pub mod cli {
  use clap::{Parser, Subcommand};
  use open::that;
  use simply_colored::*;
  use crate::APP_DIRS;

  #[derive(Debug, Parser)]
  pub enum Commands {
    Config {
      #[command(subcommand)]
      subcommands: SubCommands
    }
  }

  #[derive(Debug, Subcommand)]
  pub enum SubCommands {
    Path,
    Edit
  }

  pub fn handle(cmd: &Commands) {
    match cmd {
      Commands::Config { subcommands } => {
        match subcommands {
          SubCommands::Path => {
            println!("{DIM_YELLOW}Config path: {RESET}{BOLD}{:?}{RESET}", APP_DIRS.config_dir);
          },
          SubCommands::Edit => {
            let config_path = APP_DIRS.config_dir.join("config.toml");
            if let Err(e) = that(config_path) {
              eprintln!("{RED}Failed to open config file: {RESET}{e}");
            }
          }
        }
      }
    }
  }
}

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