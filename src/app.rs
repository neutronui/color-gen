use std::{fs, path::PathBuf};
use serde_json::from_str;
use crate::{config::{self, Cli}, css::generate_palette_css};

pub enum AppState {
  Uninitialized,
  ConfigLoaded(config::Config),
  Validated(config::Config),
  Generated(config::Config, Vec<PathBuf>),
}

pub trait StateBehavior {
  fn load_config(self, cli: &Cli) -> Result<AppState, Box<dyn std::error::Error>>;
  fn validate(self) -> Result<AppState, Box<dyn std::error::Error>>;
  fn generate_css(self) -> Result<AppState, Box<dyn std::error::Error>>;
}

impl StateBehavior for AppState {
  fn load_config(self, cli: &Cli) -> Result<AppState, Box<dyn std::error::Error>> {
    match self {
      AppState::Uninitialized => {
        let data = fs::read_to_string(&cli.config)?;
        let config: config::Config = from_str(&data)?;

        Ok(AppState::ConfigLoaded(config))
      }
      _ => Err("Config can only be loaded from Uninitialized state.".into()),
    }
  }

  fn validate(self) -> Result<AppState, Box<dyn std::error::Error>> {
    match self {
      AppState::ConfigLoaded(config) => {
        if config.themes.is_empty() {
          Err("No themes defined in configuration.".into())
        } else {
          Ok(AppState::Validated(config))
        }
      }
      _ => Err("Validation can only be performed from ConfigLoaded state.".into()),
    }
  }

  fn generate_css(self) -> Result<AppState, Box<dyn std::error::Error>> {
    match self {
      AppState::Validated(config) => {
        let mut css_files = Vec::new();

        Ok(AppState::Generated(config, css_files))
      }
      _ => Err("CSS generation can only be performed from Validated state.".into()),
    }
  }
}