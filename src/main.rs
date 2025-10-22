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

fn tonal_steps() -> [u8; 11] {
  [05, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95]
}

struct CSSColorToken {
  prefix: Option<String>,
  palette_name: String,
  tone: u8,
  value: BigColor
}

impl CSSColorToken {
  fn new(prefix: Option<String>, palette_name: String, tone: u8, value: BigColor) -> Self {
    Self {
      prefix,
      palette_name,
      tone,
      value
    }
  }

  fn to_string(&self, with_value: bool) -> String {
    let mut context = tera::Context::new();
    context.insert("prefix", &self.prefix);
    context.insert("palette_name", &self.palette_name);
    context.insert("tone", format!("{:02}", self.tone).as_str());

    let mut token = TEMPLATES.render("COLOR_TOKEN", &context).unwrap();

    if with_value {
      token = format!("{}: {};", token, self.value.to_oklch_string());
    }

    token
  }
}

struct CSSBaseToken {
  prefix: Option<String>,
  palette_name: String,
  value: String,
}

impl CSSBaseToken {
  fn new(prefix: Option<String>, palette_name: String, value: String) -> Self {
    Self {
      prefix,
      palette_name,
      value
    }
  }

  fn to_string(&self) -> String {
    let mut context = tera::Context::new();
    context.insert("prefix", &self.prefix);
    context.insert("palette_name", &self.palette_name);

    let mut token = TEMPLATES.render("COLOR_BASE", &context).unwrap();
    token = format!("{}: {};", token, self.value);

    token
  }
}

struct CSSKeyToken {
  prefix: Option<String>,
  palette_name: String,
  value: u8,
}

impl CSSKeyToken {
  fn new(prefix: Option<String>, palette_name: String, value: u8) -> Self {
    Self {
      prefix,
      palette_name,
      value
    }
  }

  fn to_string(&self) -> String {
    let mut context = tera::Context::new();
    context.insert("prefix", &self.prefix);
    context.insert("palette_name", &self.palette_name);

    let mut token = TEMPLATES.render("COLOR_KEY", &context).unwrap();
    token = format!("{}: {};", token, format!("{:02}", self.value));

    token
  }
}

fn generate_palette_css(palette: &PaletteConfig) -> Result<(), Box<dyn std::error::Error>> {

}

fn generate_variant_css(variant: &str) -> Result<(), Box<dyn std::error::Error>> {

}

fn generate_theme_css(theme: &config::ThemeConfig) -> Result<(), Box<dyn std::error::Error>> {

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let cli = Cli::parse();
  app::App::new()
    .load_config(&cli)?
    .validate()?
    .generate_css()?;

  Ok(())
}

// fn main() -> Result<()> {
//   let cli = Cli::parse();
//   init_config(&cli);

//   for theme in &CONFIG.get().unwrap().themes {
//     println!("Theme: {}", theme.name);

//     for (palette_name, palette_config) in &theme.palettes {
//       println!("  Palette: {}", palette_name);
//       println!("    Base Color: {}", palette_config.base);

//       let source_color = BigColor::new(&palette_config.base);
//       let source_scale = source_color.monochromatic(Some(tonal_steps().len()));

//       let mut color_tokens: Vec<CSSColorToken> = vec![];

//       for (index, color) in source_scale.iter().enumerate() {
//         let tone = tonal_steps()[index];
//         let token = CSSColorToken::new(
//           theme.prefix.clone(),
//           palette_name.clone(),
//           tone,
//           color.clone()
//         );

//         color_tokens.push(token);
//       }
//       let key_color = closest_to_base(&source_color, &source_scale)?;
//       let key_tone = source_scale.iter().position(|c| c == &key_color).unwrap();

//       let key_token = CSSKeyToken::new(
//         theme.prefix.clone(),
//         palette_name.clone(),
//         tonal_steps()[key_tone]
//       );

//       let base_token = CSSBaseToken::new(
//         theme.prefix.clone(),
//         palette_name.clone(),
//         key_color.to_oklch_string()
//       );
//       println!("    {}", key_token.to_string());
//       println!("    {}", base_token.to_string());


//       if let Some(variant) = &palette_config.variant {
//       }
//     }
//   }

//   Ok(())
// }

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