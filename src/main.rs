// use std::{collections::{BTreeMap, HashMap}, fs, path::{Path, PathBuf}};
// use lazy_static::lazy_static;
// use serde::{Deserialize, Serialize};

// use clap::Parser;
// use bigcolor::BigColor;
// use serde_json::from_str;
// use tera::Tera;

// const VARIANT_TEMPLATE: &str = r#"
// {% for variant in variants -%}
// {{ variant.selector }} {
//   {%- for variable in variant.variables %}
//   {{ variable }}
//   {%- endfor %}
// }
// {% endfor %}
// "#;

// const PALETTE_TEMPLATE: &str = r#"
// /*
//   {{ name }} Palette
//   {{ description }}
// */

// {% for variant_file in variant_file_names -%}
// @import("./{{ variant_file }}");
// {% endfor -%}
// {{ selector }} {
//   {%- for variable in variables %}
//   {{ variable }}
//   {%- endfor %}
// }
// "#;

// lazy_static! {
//   pub static ref TEMPLATES: Tera = {
//     let mut tera = Tera::default();

//     tera.add_raw_templates(vec![
//       ("palette.css.tera", PALETTE_TEMPLATE),
//       ("variant.css.tera", VARIANT_TEMPLATE)
//     ]).expect("Could not add templates");

//     tera
//   };

//   pub static ref APP_DATA_DIR: PathBuf = {
//     #[cfg(unix)]
//     let app_data = std::env::var("HOME").expect("No HOME directory");
//     #[cfg(windows)]
//     let app_data = std::env::var("APPDATA").expect("No APPDATA directory");

//     PathBuf::from(app_data).join(".substrate-color-gen")
//   };
// }

// #[derive(Debug, Parser)]
// #[command(name = "substrate-color-gen")]
// #[command(bin_name = "substrate")]
// #[command(about, version)]
// struct Cli {
//   #[arg(short, long, value_name = "FILE_PATH")]
//   pub config: PathBuf,

//   #[arg(short, long, action = clap::ArgAction::Count)]
//   debug: u8,
// }

// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct Config {
//   out_dir: String,
//   palettes: Vec<PaletteConfig>
// }


// #[derive(Debug, Deserialize)]
// struct PaletteConfig {
//   name: String,
//   default: Option<bool>,
//   description: Option<String>,
//   prefix: Option<String>,
//   tones: Vec<u8>,
//   hues: HashMap<String, String>,
//   variants: HashMap<String, String>
// }

// #[derive(Debug, Clone)]
// struct Scale {
//   name: String,
//   key_tone: u8,
//   shades: BTreeMap<u8, Shade>
// }

// #[derive(Debug, Clone)]
// struct Shade {
//   tone: u8,
//   color: BigColor,
//   name: String
// }

// #[derive(Debug, Serialize)]
// struct CssVariant {
//   selector: String,
//   variables: Vec<String>
// }

// #[derive(Debug, Serialize)]
// struct CssVariantCtx {
//   variants: Vec<CssVariant>
// }

// #[derive(Debug, Serialize)]
// struct CssPaletteCtx {
//   name: String,
//   description: String,
//   variant_file_names: Vec<String>,
//   selector: String,
//   variables: Vec<String>
// }

// fn normalize_out_dir(config_dir: &Path, out: &str) -> PathBuf {
//   let p = Path::new(out);
//   if p.is_absolute() {
//     p.to_path_buf()
//   } else {
//     config_dir.join(p)
//   }
// }

// fn closest_to_base(base_color: &BigColor, shades: &BTreeMap<u8, Shade>) -> u8 {
//   let base_oklch = base_color.to_oklch();
//   let closest = shades
//     .values()
//     .min_by(|a, b| {
//       (a.color.to_oklch().l - base_oklch.l)
//         .abs()
//         .partial_cmp(&(b.color.to_oklch().l - base_oklch.l).abs())
//         .unwrap()
//     })
//     .unwrap();

//   closest.clone().tone
// }

// trait StringExtensions {
//   fn with_prefix(&self, prefix: &str) -> String;
// }

// impl StringExtensions for String {
//   fn with_prefix(&self, prefix: &str) -> String {
//     if prefix.is_empty() {
//       self.clone()
//     } else {
//       format!("{}-{}", prefix, self)
//     }
//   }
// }

// fn main() {
//   let cli = Cli::parse();
//   let data = fs::read_to_string(&cli.config)
//     .expect("Failed to read config file");
//   let config: Config = from_str(&data)
//     .expect("Failed to parse config JSON");
//   let out_dir = normalize_out_dir(&cli.config.parent().unwrap_or(Path::new(".")), &config.out_dir);
//   fs::create_dir_all(&out_dir).expect("Failed to create output directory");
  
//   println!("Output directory: {:?}", out_dir);

//   for palette_cfg in &config.palettes {
//     let name = &palette_cfg.name;
//     let description = palette_cfg.description.as_deref().unwrap_or_default();
//     let is_default = palette_cfg.default.unwrap_or_default();
//     let prefix = palette_cfg.prefix.as_deref().unwrap_or_default();
//     let tones = &palette_cfg.tones;
//     let palette_out_dir = out_dir.join(name);
//     fs::create_dir_all(&palette_out_dir).expect("Failed to create palette output directory");
    
//     let mut scales = Vec::<Scale>::new();
//     let default_selector = ":where(:root)";
    
//     for (hue_name, base_color_raw) in &palette_cfg.hues {
//       let mut shades = BTreeMap::<u8, Shade>::new();
//       let base_color = BigColor::new(base_color_raw);
//       let colors = base_color.monochromatic(Some(tones.len()));

//       for (idx, color) in colors.iter().enumerate() {
//         shades.insert(tones[idx], Shade {
//           tone: tones[idx],
//           // TODO: Expose adjustments via config or CLI
//           color: color.clone().lighten(Some(5.0)).clone(),
//           name: format!("color-{}-{:02}", hue_name, tones[idx]).with_prefix(prefix)
//         });
//       }

//       let key_tone = closest_to_base(&base_color, &shades);

//       scales.push(Scale {
//         name: hue_name.clone(),
//         shades,
//         key_tone
//       });
//     }

//     let mut variant_file_names = Vec::<String>::new();

//     for (variant, hue) in &palette_cfg.variants {
//       let variant_file_name = format!("{}.css", variant);
//       let out_path = palette_out_dir.join(&variant_file_name);
//       variant_file_names.push(variant_file_name.clone());

//       let mut variants = Vec::<CssVariant>::new();
      
//       let mut ordered_scales: Vec<&Scale> = Vec::with_capacity(scales.len());
//       if let Some(pos) = scales.iter().position(|s| s.name == *hue) {
//         ordered_scales.push(&scales[pos]);
//         for (i, s) in scales.iter().enumerate() {
//           if i != pos {
//             ordered_scales.push(s);
//           }
//         }
//       } else {
//         for s in &scales {
//           ordered_scales.push(s);
//         }
//       }

//       for scale in ordered_scales {
//         let selector = if scale.name == *hue {
//           format!("{}, .{}-{}", default_selector, variant.with_prefix(prefix), scale.name)
//         } else {
//           format!(".{}-{}", variant.with_prefix(prefix), scale.name)
//         };

//         let mut css_vars = Vec::<String>::new();

//         for shade in scale.shades.values() {
//           let var_name = format!("color-{}-{:02}", variant, shade.tone).with_prefix(prefix);
//           css_vars.push(format!("{}: var(--{});", format!("--{}", var_name), shade.name));
//         }
//         let key_color_prop = format!("color-{}", scale.name).with_prefix(prefix);
//         let key_color_name = scale.shades.get(&scale.key_tone).unwrap().name.clone();
//         css_vars.push(format!("--{}: var(--{});", key_color_prop, key_color_name));
//         let key_on_prop = format!("color-{}-on", scale.name).with_prefix(prefix);
//         let key_on_name = format!("color-{}-on", scale.name).with_prefix(prefix);
//         css_vars.push(format!("--{}: var(--{});", key_on_prop, key_on_name));

//         variants.push(CssVariant {
//           selector,
//           variables: css_vars
//         });
//       }

//       let ctx = tera::Context::from_serialize(CssVariantCtx {
//         variants
//       }).unwrap();
      
//       println!("Generated variant file: {:?}", out_path);
//       let rendered = TEMPLATES.render("variant.css.tera", &ctx).unwrap();
//       fs::write(&out_path, rendered).expect("Failed to write output file");
//     }

//     let palette_file_name = format!("{}.css", &palette_cfg.name);
//     let selector = if is_default {
//           format!("{}, .{}-palette", default_selector, &palette_cfg.name.with_prefix(prefix))
//         } else {
//           format!(".{}-palette", &palette_cfg.name.with_prefix(prefix))
//         };
//     let mut css_vars = Vec::<String>::new();
    
//     for scale in &scales {
//       for shade in scale.shades.values() {
//         css_vars.push(format!("--{}: {};", shade.name, shade.color.to_oklch_string()));
//       }

//       let key_color_prop = format!("color-{}", scale.name).with_prefix(prefix);
//       let key_color_name = scale.shades.get(&scale.key_tone).unwrap().name.clone();
//       let key_prop = format!("{}-key", scale.name).with_prefix(prefix);

//       css_vars.push(format!("--{}: var(--{});", key_color_prop, key_color_name));
//       css_vars.push(format!("--{}: {};", key_prop, scale.key_tone));
//       if !std::ptr::eq(scale, scales.last().unwrap()) {
//         css_vars.push(String::new());
//       }
//     }

//     let ctx = tera::Context::from_serialize(CssPaletteCtx {
//       name: name.clone(),
//       description: description.to_string(),
//       variant_file_names,
//       selector,
//       variables: css_vars
//     }).unwrap();


//     let out_path = palette_out_dir.join(&palette_file_name);
//     println!("Generated palette file: {:?}", out_path);
//     let rendered = TEMPLATES.render("palette.css.tera", &ctx).unwrap();
//     fs::write(&out_path, rendered).expect("Failed to write output file");
//   }
// }

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use platform_dirs::{AppDirs, UserDirs};
use lazy_static::lazy_static;
use simply_colored::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Commands,

  // #[arg(short, long)]
  // config: PathBuf,

  #[arg(long)]
  cwd: Option<PathBuf>,

  #[arg(short = 'o', long = "out")]
  out_dir: Option<PathBuf>,

  #[arg(short, long)]
  watch: Option<bool>,

  #[arg(short, long)]
  quiet: Option<bool>,

  #[arg(short, long)]
  verbose: Option<bool>,

  #[arg(long = "no-color")]
  no_color: Option<bool>,

  #[arg(long = "dry-run")]
  dry_run: Option<bool>,

  #[arg(long = "no-cache")]
  no_cache: Option<bool>,
}

#[derive(Subcommand)]
enum Commands {
  #[command(about = "Manage themes")]
  Theme { },

  #[command(about = "Manage tokens")]
  Tokens { },

  #[command(about = "Manage configuration")]
  Config {
    #[command(subcommand)]
    commands: ConfigCommands,
  },
}

#[derive(Subcommand)]
enum ConfigCommands {
  #[command(about = "Show current config path")]
  Path,

  #[command(about = "Open config for editing")]
  Edit { },

  #[command()]
  Get { },

  #[command()]
  Set { },
}

lazy_static! {
  pub static ref APP_DIRS: AppDirs = AppDirs::new(Some("substrate"), false).unwrap();
  pub static ref USER_DIRS: UserDirs = UserDirs::new().unwrap();
}

fn ensure_config_dir(path: &PathBuf) {
  if !path.exists() {
    std::fs::create_dir_all(path).expect("Could not create config directory");
  }
}

fn hyperlink(link: impl core::fmt::Display, text: impl core::fmt::Display) -> String {
  format!("\x1b]8;;{link}\x1b\\{text}\x1b]8;;\x1b\\")
}

fn main() {
  let cli = Cli::parse();

  ensure_config_dir(&APP_DIRS.config_dir);

  match &cli.command {
    Commands::Theme { } => {
      todo!()
    },
    Commands::Tokens { } => {
      todo!()
    },
    Commands::Config { commands } => {
      match commands {
        ConfigCommands::Path => {
          println!("{DIM_YELLOW}Config path: {RESET}{BOLD}{:?}{RESET}", APP_DIRS.config_dir);
        },
        ConfigCommands::Edit {} => {
          todo!()
        },
        ConfigCommands::Get {} => {
          todo!()
        },
        ConfigCommands::Set {} => {
          todo!()
        }
      }
    }
  }
}