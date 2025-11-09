use crate::config;


pub mod cli {
  use clap::{Parser, Subcommand};
  use simply_colored::*;
  
  #[derive(Debug, Parser)]
  pub enum Commands {
    Tokens {
      #[command(subcommand)]
      subcommands: SubCommands
    }
  }

  #[derive(Debug, Subcommand)]
  enum SubCommands {
    Transform,
  }

  pub fn handle(cmd: &Commands) {
    match cmd {
      Commands::Tokens { subcommands } => {
        match subcommands {
          SubCommands::Transform => {
            println!("{DIM_MAGENTA}Transforming tokens...{RESET}");
            
          },
        }
      }
    }
  }
}

fn resolve_config() -> config::Config {
  todo!();
}

fn find_duplicates(tokens: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut duplicates = std::collections::HashSet::new();

    for token in tokens {
        if !seen.insert(token.clone()) {
            duplicates.insert(token);
        }
    }

    duplicates.into_iter().collect()
}

fn resolve_references() {
  todo!()
}