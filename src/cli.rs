use core::fmt;

use clap::{ArgMatches, Args, Command, FromArgMatches, Subcommand};

pub struct Cli {
  pub command: Command,
  handlers: Vec<Box<dyn Fn(&ArgMatches) + Send + Sync>>
}

impl fmt::Debug for Cli {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Cli")
      .field("command", &self.command)
      .field("handlers_count", &self.handlers.len())
      .finish()
  }
}

impl Cli {
  pub fn new() -> Self {
    let command = Command::new(env!("CARGO_PKG_NAME"))
      .version(env!("CARGO_PKG_VERSION"))
      .author(env!("CARGO_PKG_AUTHORS"))
      .about(env!("CARGO_PKG_DESCRIPTION"));

    Self {
      command,
      handlers: Vec::new()
    }
  }

  pub fn register_commands<T: Subcommand>(&mut self, handler: fn(&T)) -> &Self where T: Subcommand + FromArgMatches + 'static {
    self.command = T::augment_subcommands(self.command.clone());
    self.handlers.push(Box::new(move |matches: &ArgMatches| {
      if let Ok(cmd) = T::from_arg_matches(matches) {
        handler(&cmd)
      }
    }));
    self
  }

  pub fn register_args<T: Args>(&mut self) -> &Self {
    self.command = T::augment_args(self.command.clone());
    self
  }

  pub fn parse_and_dispatch(&self) -> ArgMatches {
    let matches = self.command.clone().get_matches();
    for h in &self.handlers {
      h(&matches);
    }
    matches
  }
}