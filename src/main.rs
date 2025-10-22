use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use anyhow::{bail, Context as A_Context, Result};
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use bigcolor::{BigColor, color_space::{OKLCH}};
use serde_json::from_str;
use tera::Tera;

use crate::config::{Cli, PaletteConfig};

mod config;
mod app;
mod css;

lazy_static! {
  pub static ref TEMPLATES: Tera = {
    let mut tera = match Tera::new("templates/*.tera") {
      Ok(t) => t,
      Err(e) =>  {
        println!("Parsing error(s): {}", e);
        std::process::exit(1);
      }
    };

    tera.add_raw_template("COLOR_TOKEN", "--{% if prefix %}{{ prefix }}-{% endif %}color-{{ palette_name }}-{{ tone }}").unwrap();
    tera.add_raw_template("COLOR_BASE", "--{% if prefix %}{{ prefix }}-{% endif %}color-{{ palette_name }}").unwrap();
    tera.add_raw_template("COLOR_KEY", "--{% if prefix %}{{ prefix }}-{% endif %}color-{{ palette_name }}-key").unwrap();

    tera
  };
}

// fn generate_variant_css(variant: &str) -> Result<(), Box<dyn std::error::Error>> {

// }

// fn generate_theme_css(theme: &config::ThemeConfig) -> Result<(), Box<dyn std::error::Error>> {

// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let cli = Cli::parse();
  app::App::new()
    .load_config(&cli)?
    .validate()?
    .generate_css()?;

  Ok(())
}