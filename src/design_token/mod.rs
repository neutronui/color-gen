use clap::Subcommand;
use simply_colored::*;
use crate::APP_DIRS;

mod css_var;
mod resolver;
mod token;

#[derive(Subcommand)]
pub enum DesignTokenCommands {

}

pub fn handle_design_token_commands(commands: &DesignTokenCommands) {
  match commands {
    
  }
}