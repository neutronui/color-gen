use clap::Subcommand;
use simply_colored::*;
use crate::APP_DIRS;

mod css_var;
mod resolver;
mod token;

#[derive(Subcommand)]
pub enum DesignTokenCommands {
  Import,
}

pub fn handle_design_token_commands(commands: &DesignTokenCommands) {
  match commands {
    DesignTokenCommands::Import => {
      todo!()
    }
  }
}