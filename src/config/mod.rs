use clap::Subcommand;
use simply_colored::*;
use open::that;
use crate::APP_DIRS;

mod utils;

#[derive(Subcommand)]
pub enum ConfigCommands {
  #[command(about = "Show current config path")]
  Path,

  #[command(about = "open config file in default editor")]
  Edit,
}

pub fn handle_config_commands(commands: &ConfigCommands) {
  utils::ensure_config();

  match commands {
    ConfigCommands::Path => {
      println!("{DIM_YELLOW}Config path: {RESET}{BOLD}{:?}{RESET}", APP_DIRS.config_dir);
    },
    ConfigCommands::Edit => {
      let config_path = APP_DIRS.config_dir.join("config.toml");
      if let Err(e) = that(config_path) {
        eprintln!("{RED}Failed to open config file: {RESET}{e}");
      }
    }
  }
}