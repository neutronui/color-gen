use std::path::PathBuf;

use clap::{Args};
use platform_dirs::{AppDirs, UserDirs};
use lazy_static::lazy_static;

mod cli;
mod config;
mod transformer;
mod utils;

use cli::Cli;

lazy_static! {
  pub static ref APP_DIRS: AppDirs = AppDirs::new(Some("substrate"), false)
    .expect("Failed to get application directories");
  pub static ref USER_DIRS: UserDirs = UserDirs::new()
    .expect("Failed to get user directories");
}

#[derive(Args)]
struct CliArgs {
  #[arg(long, value_name = "PATH")]
  cwd: Option<PathBuf>,

  #[arg(short = 'o', long= "out", value_name = "PATH")]
  out_dir: Option<PathBuf>,

  #[arg(short, long, default_value_t = false)]
  watch: bool,

  #[arg(short, long, default_value_t = false)]
  quiet: bool,

  #[arg(short, long, default_value_t = false)]
  verbose: bool,

  #[arg(long = "no-color", default_value_t = false)]
  no_color: bool,

  #[arg(long = "dry-run", default_value_t = false)]
  dry_run: bool,

  #[arg(long = "no-cache", default_value_t = false)]
  no_cache: bool
}

mod test {
  use clap::{Parser};

  #[derive(Debug, Parser)]
  pub enum Commands {
    Foo {
      #[arg(short, long)]
      bar: String,
    }
  }

  pub fn handle(cmd: &Commands) {
    match cmd {
      Commands::Foo { bar } => {
        println!("Foo command executed with bar: {}", bar);
      }
    }
  }
}

fn main() {
  let mut cli = Cli::new();
  cli.register_args::<CliArgs>();
  cli.register_commands::<test::Commands, _>(test::handle);
  cli.register_commands::<config::cli::Commands, _>(config::cli::handle);

  let matches = cli.command.get_matches();
  let is_watching = matches.get_flag("watch");

  println!("Is watching: {}", is_watching);
}