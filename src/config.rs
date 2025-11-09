use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};

pub mod cli {
  use std::path::PathBuf;
  use clap::{Parser, Subcommand};
  use open::that;
  use simply_colored::*;
  use crate::{APP_DIRS, config::config_from};

  #[derive(Debug, Parser)]
  pub enum Commands {
    Config {
      #[command(subcommand)]
      subcommands: SubCommands,

      #[arg(name = "file", long, value_name = "PATH")]
      file: Option<PathBuf>,
    },
  }

  #[derive(Debug, Subcommand)]
  enum SubCommands {
    Path,
    Edit
  }

  pub fn handle(cmd: &Commands) {
    match cmd {
      Commands::Config { subcommands, file } => {
        if let Some(path) = file {
          match config_from(path) {
            Ok(_) => println!("{DIM_GREEN}Config file successfully loaded from {:?}{RESET}", path),
            Err(e) => eprintln!("{BG_RED}Failed to load config from {:?}: {RESET}{e}", path),
          }
        }
        
        match subcommands {
          SubCommands::Path => {
            println!("{DIM_GREEN}Config path{RESET}{BOLD} => {:?}{RESET}", APP_DIRS.config_dir);
          },
          SubCommands::Edit => {
            let config_path = APP_DIRS.config_dir.join("config.toml");
            if let Err(e) = that(config_path) {
              eprintln!("{BG_RED}Failed to open config file: {RESET}{e}");
            }
          }
        }
      }
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
  pub transforms: Vec<Transform>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
  pub from: PathBuf,
  pub to: Vec<TransformTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformTarget {
  pub format: TargetFormat,
  pub output: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetFormat {
  Json,
  Toml,
  Yaml,
  Scss,
  Css,
  Mjs
}

pub fn load_config(path: &PathBuf) -> Option<Config> {
  let config_str = std::fs::read_to_string(path).ok()?;
  let config: Config = toml::from_str(&config_str).ok()?;
  Some(config)
}

pub fn config_from<P: AsRef<std::path::Path>>(path: P) -> Result<(), String> {

  fn detect_format(path: &Path) -> Result<&'static str, String> {
    let ext = path
      .extension()
      .and_then(|s| s.to_str())
      .map(|s| s.to_ascii_lowercase())
      .ok_or_else(|| "Input file has no extension; expected .json, .yaml/.yml, or .toml".to_string())?;

    match ext.as_str() {
      "json" => Ok("json"),
      "yaml" | "yml" => Ok("yaml"),
      "toml" => Ok("toml"),
      _ => Err(format!("Unsupported file extension '.{}'; expected .json, .yaml/.yml, or .toml", ext)),
    }
  }

  fn validate_config(cfg: &Config) -> Result<(), String> {
    for (i, t) in cfg.transforms.iter().enumerate() {
      if t.to.is_empty() {
        return Err(format!("Transform at index {i} must have at least one target"));
      }
      if !t.from.exists() {
        return Err(format!("Transform at index {i} refers to non-existent 'from' path: {:?}", t.from));
      }
      for (j, target) in t.to.iter().enumerate() {
        // Basic sanity: output must be non-empty path
        if target.output.as_os_str().is_empty() {
          return Err(format!("Transform[{i}].to[{j}] has an empty output path"));
        }
        // No further checks on output; it might be created later by other code.
        let _ = &target.format; // format presence guaranteed by deserialization
      }
    }
    Ok(())
  }

  let path = path.as_ref();
  let input = fs::read_to_string(path)
    .map_err(|e| format!("Failed to read input file {:?}: {}", path, e))?;

  let fmt = detect_format(path)?;
  let cfg: Config = match fmt {
    "json" => serde_json::from_str(&input).map_err(|e| format!("Failed to parse JSON: {e}"))?,
    "yaml" => serde_yaml::from_str(&input).map_err(|e| format!("Failed to parse YAML: {e}"))?,
    "toml" => toml::from_str(&input).map_err(|e| format!("Failed to parse TOML: {e}"))?,
    _ => unreachable!(),
  };

  validate_config(&cfg)?;

  // Persist to internal TOML configuration file
  let config_dir = crate::APP_DIRS.config_dir.clone();
  fs::create_dir_all(&config_dir)
    .map_err(|e| format!("Failed to create config directory {:?}: {}", config_dir, e))?;

  let dest = config_dir.join("config.toml");
  let toml_str = toml::to_string_pretty(&cfg)
    .map_err(|e| format!("Failed to serialize config to TOML: {e}"))?;

  fs::write(&dest, toml_str)
    .map_err(|e| format!("Failed to write config to {:?}: {}", dest, e))?;

  Ok(())
}