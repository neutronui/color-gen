use std::path::PathBuf;
use serde::{Serialize, Deserialize};

pub mod cli {
  use clap::Subcommand;
  use open::that;

  #[derive(Subcommand)]
  pub enum Commands {
    #[command(about = "Show current config path")]
    Path,
    #[command(about = "Open configuration file with default text editor")]
    Edit,
  }

  trait ConfigCommands {}
  impl ConfigCommands for Commands {

  }
}