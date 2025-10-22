use std::path::{PathBuf};

use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug)]
pub struct Cli {
  pub config: PathBuf
}

#[derive(Deserialize, Debug)]
pub struct PaletteConfig {
  pub base: String,
  pub variant: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct ThemeConfig {
  pub name: String,
  pub default: Option<bool>,
  pub prefix: Option<String>,
  pub description: Option<String>,
  pub palettes: std::collections::BTreeMap<String, PaletteConfig>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  pub out_dir: String,
  pub themes: Vec<ThemeConfig>
}