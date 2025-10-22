use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::Parser;
use anyhow::{bail, Context as A_Context, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use bigcolor::{BigColor, color_space::{OKLCH}};
use tera::{Tera, Context as T_Context};
use serde_json::from_str;

#[derive(Parser, Debug)]
struct Cli {
  config: PathBuf
}

#[derive(Deserialize, Debug)]
struct PaletteConfig {
  base: String,
  variant: Option<String>
}

#[derive(Deserialize, Debug)]
struct ThemeConfig {
  name: String,
  default: Option<bool>,
  prefix: Option<String>,
  description: Option<String>,
  palettes: std::collections::BTreeMap<String, PaletteConfig>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Config {
  out_dir: String,
  themes: Vec<ThemeConfig>
}

#[derive(Serialize, Debug)]
struct Palette {
  name: String,
  base: (String, String),
  key: (String, String),
  tokens: std::collections::BTreeMap<u8, (String, String)>
}

const COLOR_TOKEN_TEMPLATE: &str = "--{% if prefix %}{{ prefix }}-{% endif %}color-{{ palette_name }}-{{ tone }}";
const COLOR_BASE_TEMPLATE: &str = "--{% if prefix %}{{ prefix }}-{% endif %}color-{{ palette_name }}";
const COLOR_KEY_TEMPLATE: &str = "--{% if prefix %}{{ prefix }}-{% endif %}color-{{ palette_name }}-key";

fn tonal_steps() -> Vec<u8> {
  vec![05, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95]
}

fn main() -> Result<()> {
  let cli = Cli::parse();
  let config_path = cli.config.canonicalize()?;
  let config_dir = config_path.parent().unwrap_or(Path::new("."));
  let config_raw = fs::read_to_string(&config_path)?;
  let config: Config = from_str(&config_raw)?;
  let mut tera = Tera::new("templates/*.tera")?;

  tera.add_raw_templates([
    ("COLOR_TOKEN", COLOR_TOKEN_TEMPLATE),
    ("COLOR_BASE", COLOR_BASE_TEMPLATE),
    ("COLOR_KEY", COLOR_KEY_TEMPLATE)
  ])?;

  for theme in config.themes {
    let name = theme.name;
    let is_default = theme.default.unwrap_or(false);
    let prefix = theme.prefix.unwrap_or_default();
    let description = theme.description.unwrap_or_default();
    
    for (palette_name, palette_config) in theme.palettes {
      let base_color = BigColor::new(palette_config.base);
      let for_variant = palette_config.variant.unwrap();

      let mut palette_context = T_Context::new();
      palette_context.insert("prefix", prefix.as_str());
      palette_context.insert("palette_name", palette_name.as_str());

      let scale = base_color.monochromatic(Some(tonal_steps().len()));
      let key_color = closest_to_base(&base_color, &scale)?;

      // let mut palette = Palette {
      //   base: (
      //     tera.render("COLOR_BASE", &palette_context)?.to_string(),
      //     base_color.to_hex_string(false)
      //   )
      // };
    }
  }

  Ok(())
}

fn closest_to_base(base: &BigColor, palette: &Vec<BigColor>) -> anyhow::Result<BigColor> {
  let base_oklch = base.to_oklch();
  let closest = palette
    .iter()
    .min_by(|a, b| {
      (a.to_oklch().l - base_oklch.l)
        .abs()
        .partial_cmp(&(b.to_oklch().l - base_oklch.l).abs())
        .unwrap()
    })
    .unwrap_or(palette.get(palette.len() / 2).unwrap());

  Ok(closest.clone())
}

fn collect_palette_tokens() -> anyhow::Result<()> {
  Ok(())
}