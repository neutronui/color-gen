use clap::{ArgMatches, Args, Command, FromArgMatches, Subcommand};

#[derive(Debug)]
pub struct Cli {
  pub command: Command
}

impl Cli {
  pub fn new() -> Self {
    let command = Command::new(env!("CARGO_PKG_NAME"))
      .version(env!("CARGO_PKG_VERSION"))
      .author(env!("CARGO_PKG_AUTHORS"))
      .about(env!("CARGO_PKG_DESCRIPTION"));

    Self {
      command
    }
  }

  pub fn register_commands<T: Subcommand, F>(&mut self, handler: F) -> &Self where F: for<'a> Fn(&'a T) {
    let cli = T::augment_subcommands(self.command.clone());
    let matches = cli.clone().get_matches();
    handler(&extract::<T>(&matches).unwrap());
    
    self.command = cli;
    self
  }

  pub fn register_args<T: Args>(&mut self) -> &Self {
    let cli = T::augment_args(self.command.clone());
    
    self.command = cli;
    self
  }
}

fn extract<T>(matches: &ArgMatches) -> Option<T> where T: Subcommand + FromArgMatches {
  T::from_arg_matches(matches)
    .map_err(|err| err.exit())
    .ok()
}