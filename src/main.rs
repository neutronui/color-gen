// use std::collections::HashMap;
// use std::fs;
// use std::path::{Path, PathBuf};

// use clap::Parser;
// use itertools::Itertools;
// use lazy_static::lazy_static;
// use serde::{Deserialize, Serialize};
// use bigcolor::{BigColor, color_space::{OKLCH}};
// use serde_json::from_str;
// use tera::Tera;

// use crate::config::{Cli, PaletteConfig};
// use crate::app::{StateBehavior};

// mod config;
// mod app;
// mod css;

// lazy_static! {
//   pub static ref TEMPLATES: Tera = {
//     let mut tera = match Tera::new("templates/*.tera") {
//       Ok(t) => t,
//       Err(e) =>  {
//         println!("Parsing error(s): {}", e);
//         std::process::exit(1);
//       }
//     };

//     tera.add_raw_template("COLOR_TOKEN", "--{% if prefix %}{{ prefix }}-{% endif %}color-{{ palette_name }}-{{ tone }}").unwrap();
//     tera.add_raw_template("COLOR_BASE", "--{% if prefix %}{{ prefix }}-{% endif %}color-{{ palette_name }}").unwrap();
//     tera.add_raw_template("COLOR_KEY", "--{% if prefix %}{{ prefix }}-{% endif %}color-{{ palette_name }}-key").unwrap();

//     tera
//   };
// }

// // fn generate_variant_css(variant: &str) -> Result<(), Box<dyn std::error::Error>> {

// // }

// // fn generate_theme_css(theme: &config::ThemeConfig) -> Result<(), Box<dyn std::error::Error>> {

// // }

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//   let cli = Cli::parse();
//   let mut app_state = app::AppState::Uninitialized;

//   app_state = app_state.load_config(&cli)?;
//   app_state = app_state.validate()?;
//   app_state = app_state.generate_css()?;

//   if let app::AppState::Generated(_, css_files) = app_state {
//     println!("Generated CSS files:");
//     for file in css_files {
//       println!("- {}", file.display());
//     }
//   }


//   Ok(())
// }


use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use clap::Parser;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::from_str;
use tera::Tera;

lazy_static! {
  static ref TEMPLATES: Tera = {
    let mut tera = match Tera::new("templates/*.tera") {
      Ok(t) => t,
      Err(e) => {
        println!("Parsing error(s): {}", e);
        std::process::exit(1);
      }
    };
    
    tera
  };
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
  #[arg(short, long, value_name = "FILE_PATH")]
  pub config: PathBuf,

  #[arg(short, long, action = clap::ArgAction::Count)]
  debug: u8,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  pub out_dir: String,
  pub themes: Vec<ThemeConfig>
}

#[derive(Debug, Deserialize)]
pub struct ThemeConfig {
  pub name: String,
  pub default: Option<bool>,
  pub description: Option<String>,
  pub prefix: Option<String>,
  pub palettes: HashMap<String, PaletteConfig>
}

#[derive(Debug, Deserialize)]
pub struct PaletteConfig {
  pub base: String,
  pub variant: String
}

#[derive(Debug)]
pub struct Theme {
  pub name: String,
  pub default: bool,
  pub palettes: Vec<Palette>
}

#[derive(Debug)]
pub struct Palette {
  pub name: String,
  pub variant: String,
  pub tones: Vec<u8>,
  pub is_variant_default: bool
}

pub enum CssGenState {
  Raw(RawConfig),
  Validated(ValidatedConfig),
  Palettes(GeneratedPalettes),
  PaletteCss(PaletteCssReady),
  VariantCss(VariantCssReady),
}

pub struct RawConfig(pub Config);
pub struct ValidatedConfig(pub Config);
pub struct GeneratedPalettes(pub Vec<Theme>);
pub struct PaletteCssReady(pub Vec<Theme>);
pub struct VariantCssReady();

pub trait Validate {
  fn validate(self) -> ValidatedConfig;
}

pub trait GeneratePalettes {
  fn generate_palettes(self) -> GeneratedPalettes;
}

pub trait EmitPaletteCss {
  fn emit_palette_css(self, out_dir: &str) -> PaletteCssReady;
}

pub trait EmitVariantCss {
  fn emit_variant_css(self, out_dir: &str) -> VariantCssReady;
}

impl Validate for RawConfig {
  fn validate(self) -> ValidatedConfig {
    ValidatedConfig(self.0)
  }
}

impl GeneratePalettes for ValidatedConfig {
  fn generate_palettes(self) -> GeneratedPalettes {
    let mut themes = Vec::new();

    for theme_cfg in self.0.themes {
      let mut palettes = Vec::new();

      for (name, palette_cfg) in theme_cfg.palettes {
        let tones = vec![05, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95];
        let palette = Palette {
          name: name.clone(),
          variant: palette_cfg.variant,
          tones,
          is_variant_default: theme_cfg.default.unwrap_or(false)
        };

        palettes.push(palette);
      }

      themes.push(Theme {
        name: theme_cfg.name,
        default: theme_cfg.default.unwrap_or(false),
        palettes
      });
    }

    GeneratedPalettes(themes)
  }
}

impl EmitPaletteCss for GeneratedPalettes {
  fn emit_palette_css(self, out_dir: &str) -> PaletteCssReady {
    for theme in &self.0 {
      let mut css = String::new();
      css.push_str(&format!(".palette-{} {{\n", theme.name));

      for palette in &theme.palettes {
        for tone in &palette.tones {
          css.push_str(&format!(
            "--color-{}-{:02}: #000000;\n",
            palette.name, tone
          ));
        }
      }

      css.push_str("}\n");
      std::fs::write(format!("{}/{}.css", out_dir, theme.name), css)
        .expect("Failed to write css file");
    }

    PaletteCssReady(self.0)
  }
}

impl EmitVariantCss for PaletteCssReady {
  fn emit_variant_css(self, out_dir: &str) -> VariantCssReady {
    let mut variants: HashMap<String, Vec<&Palette>> = HashMap::new();

    for theme in &self.0 {
      for palette in &theme.palettes {
        variants.entry(palette.variant.clone())
          .or_default()
          .push(palette);
      }
    }

    
    for (variant, palettes) in &variants {
      let mut css = String::new();
      for palette in palettes {
        let selector = if palette.is_variant_default {
          format!(":where(:root),\n .{}-{} {{\n", variant, palette.name)
        } else {
          format!(".{}-{} {{\n", variant, palette.name)
        };

        css.push_str(&selector);
        
        for tone in &palette.tones {
          css.push_str(&format!(
            "--color-{}-{:02}: var(--color-{}-{:02});\n",
            variant, tone, palette.name, tone
          ));
        }

        css.push_str(&format!(
          "--color-{}: var(--color-{});\n",
          variant, palette.name
        ));
        css.push_str(&format!(
          "--color-{}-on: var(--color-{}-on);\n",
          variant, palette.name
        ));
        css.push_str("}\n\n");
      }

      std::fs::write(format!("{}/{}.css", out_dir, variant), css)
        .expect("Failed to write variant css file");
    }


    VariantCssReady()
  }
}

fn normalize_out_dir(config_dir: &Path, out: &str) -> PathBuf {
  let p = Path::new(out);
  if p.is_absolute() {
    p.to_path_buf()
  } else {
    config_dir.join(p)
  }
}

fn main() {
  let cli = Cli::parse();
  let data = fs::read_to_string(&cli.config).unwrap();
  let config: Config = from_str(&data).unwrap();
  let raw: RawConfig = RawConfig(config);
  let config_dir = &cli.config.parent().unwrap_or(Path::new("."));
  let out_dir = normalize_out_dir(config_dir, &raw.0.out_dir);
  fs::create_dir_all(&out_dir).expect("Failed to create output directory");

  
  let validated = raw.validate();
  let palettes = validated.generate_palettes();
  let palettes_ready = palettes.emit_palette_css(&out_dir.to_str().unwrap());
  let _variants_ready = palettes_ready.emit_variant_css(&out_dir.to_str().unwrap());
}