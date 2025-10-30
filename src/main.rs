use std::{collections::{BTreeMap, HashMap, HashSet}, fs, path::{Path, PathBuf}};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use clap::Parser;
use bigcolor::BigColor;
use serde_json::from_str;
use tera::Tera;

lazy_static! {
  pub static ref TEMPLATES: Tera = {
    let tera = match Tera::new("templates/*.tera") {
      Ok(t) => t,
      Err(e) => {
        println!("Parsing error(s): {}", e);
        std::process::exit(1);
      }
    };

    tera
  };
}

#[derive(Debug, Parser)]
#[command(about, version)]
struct Cli {
  #[arg(short, long, value_name = "FILE_PATH")]
  pub config: String,

  #[arg(short, long, action = clap::ArgAction::Count)]
  debug: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
  out_dir: String,
  palettes: Vec<PaletteConfig>
}


#[derive(Debug, Deserialize)]
struct PaletteConfig {
  name: String,
  default: Option<bool>,
  description: Option<String>,
  prefix: Option<String>,
  tones: Vec<u8>,
  hues: HashMap<String, String>,
  variants: HashMap<String, String>
}

#[derive(Debug, Clone)]
struct Scale {
  name: String,
  key_tone: u8,
  shades: BTreeMap<u8, Shade>
}

#[derive(Debug, Clone)]
struct Shade {
  tone: u8,
  color: BigColor,
  name: String
}

#[derive(Debug, Clone)]
struct Variant {
  name: String,
  default_hue: String,
  scales: BTreeMap<String, Scale>
}

#[derive(Debug, Clone)]
struct Palette {
  name: String,
  description: Option<String>,
  prefix: Option<String>,
  scales: HashMap<String, Scale>,
  variants: HashMap<String, Variant>
}

#[derive(Debug, Serialize)]
struct CssVariant {
  selector: String,
  variables: Vec<String>
}

#[derive(Debug, Serialize)]
struct CssVariantCtx {
  variants: Vec<CssVariant>
}

fn normalize_out_dir(config_dir: &Path, out: &str) -> PathBuf {
  let p = Path::new(out);
  if p.is_absolute() {
    p.to_path_buf()
  } else {
    config_dir.join(p)
  }
}

fn closest_to_base(base_color: &BigColor, shades: &BTreeMap<u8, Shade>) -> u8 {
  let base_oklch = base_color.to_oklch();
  let closest = shades
    .values()
    .min_by(|a, b| {
      (a.color.to_oklch().l - base_oklch.l)
        .abs()
        .partial_cmp(&(b.color.to_oklch().l - base_oklch.l).abs())
        .unwrap()
    })
    .unwrap();

  closest.clone().tone
}

fn main() {
  let cli = Cli::parse();
  let data = fs::read_to_string(&cli.config)
    .expect("Failed to read config file");
  let config: Config = from_str(&data)
    .expect("Failed to parse config JSON");
  let out_dir = normalize_out_dir(Path::new(&cli.config).parent().unwrap(), &config.out_dir);
  
  println!("Output directory: {:?}", out_dir);

  for palette_cfg in &config.palettes {
    let name = &palette_cfg.name;
    let description = palette_cfg.description.as_deref().unwrap_or_default();
    let is_default = palette_cfg.default.unwrap_or_default();
    let prefix = palette_cfg.prefix.as_deref().unwrap_or_default();
    let tones = &palette_cfg.tones;
    
    let mut scales = Vec::<Scale>::new();
    
    for (hue_name, base_color_raw) in &palette_cfg.hues {
      let mut shades = BTreeMap::<u8, Shade>::new();
      let base_color = BigColor::new(base_color_raw);
      let colors = base_color.monochromatic(Some(tones.len()));

      for (idx, color) in colors.iter().enumerate() {
        shades.insert(tones[idx], Shade {
          tone: tones[idx],
          // TODO: Expose adjustments via config or CLI
          color: color.clone().lighten(Some(5.0)).clone(),
          name: format!("color-{}-{:02}", hue_name, tones[idx])
        });
      }

      let key_tone = closest_to_base(&base_color, &shades);

      scales.push(Scale {
        name: hue_name.clone(),
        shades,
        key_tone
      });
    }

    for (variant, hue) in &palette_cfg.variants {
      let variant_file_name = format!("{}.css", variant);
      let out_path = out_dir.join(&variant_file_name);
      let default_selector = ":where(:root)";

      let mut variants = Vec::<CssVariant>::new();
      
      for scale in &scales {
        let selector = if scale.name == *hue {
          format!("{}, .{}-{}", default_selector, variant, scale.name)
        } else {
          format!(".{}-{}", variant, scale.name)
        };

        let mut css_vars = Vec::<String>::new();

        for shade in scale.shades.values() {
          let var_name = if prefix.is_empty() {
            format!("--color-{}-{}", variant, shade.tone)
          } else {
            format!("--{}-color-{}-{}", prefix, variant, shade.tone)
          };

          css_vars.push(format!("{}: var(--{});", var_name, shade.name));
        }

        variants.push(CssVariant {
          selector,
          variables: css_vars
        });
      }

      let ctx = tera::Context::from_serialize(CssVariantCtx {
        variants
      }).unwrap();
      
      let rendered = TEMPLATES.render("variant.css.tera", &ctx).unwrap();
      fs::create_dir_all(&out_path).expect("Failed to create output directory");
      fs::write(&out_path, rendered).expect("Failed to write output file");
      println!("Generated variant file: {:?}", out_path);
    }
  }
}