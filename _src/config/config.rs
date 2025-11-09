use std::path::PathBuf;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
  pub transforms: Vec<Transform>,
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