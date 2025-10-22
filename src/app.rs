use std::{fs, path::PathBuf};
use serde_json::from_str;
use crate::config::{self, Cli};

pub struct App<State> {
  state: State
}

struct Uninitialized;
struct ConfigLoaded {
  config: config::Config
}

struct Validated {
  config: config::Config
}

struct Generated {
  config: config::Config,
  css_files: Vec<PathBuf>
}

impl App<Uninitialized> {
  pub fn new() -> Self {
    App { state: Uninitialized }
  }

  pub fn load_config(self, cli: &Cli) -> Result<App<ConfigLoaded>, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(&cli.config)?;
    let config: config::Config = from_str(&data)?;
    
    Ok(App {
      state: ConfigLoaded { config }
    })
  }
}

impl App<ConfigLoaded> {
  pub fn validate(self) -> Result<App<Validated>, &'static str> {
    if self.state.config.themes.is_empty() {
      return Err("No themes defined in configuration.");
    }

    Ok(App {
      state: Validated { config: self.state.config }
    })
  }
}

impl App<Validated> {
  pub fn generate_css(self) -> Result<App<Generated>, Box<dyn std::error::Error>> {
    let mut css_files = Vec::new();

    // TODO: Implement CSS generation logic

    Ok(App {
      state: Generated {
        config: self.state.config,
        css_files
      }
    })
  }
}