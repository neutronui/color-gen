use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, from_str};
use tera::{Tera, Context};
use std::{fmt::format, fs, ops::Add, path::{Path, PathBuf}};
use color::{
  parse_color,
  OpaqueColor,
  DynamicColor,
  Oklch,
  Srgb,
  DisplayP3
};

#[derive(Parser, Debug)]
#[command(author, version, about = "", long_about = None)]
struct Cli {
  config: PathBuf
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Config {
  out_dir: String,
  themes: Vec<ThemeConfig>
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ThemeConfig {
  name: String,
  description: String,
  // #[serde(default)]
  default: bool,
  prefix: Option<String>,
  color_space: Vec<String>,
  tones: Vec<u8>,
  palettes: Map<String, Value>
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
struct ThemeContext {
  selector: String,
  color_spaces: Vec<ColorspaceContext>,
  palettes: Vec<PaletteContext>
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
struct ColorspaceContext {
  name: String,
  selector: String,
  tokens: Vec<String>
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
struct PaletteContext {
  name: String,
  tokens: Map<String, Value>
}

const COLOR_TOKEN_TEMPLATE: &str = "--{% if prefix %}{{ prefix }}-{% endif %}-{{ tone }}: {{ color_value }};";
const COLOR_BASE_TEMPLATE: &str = "--{% if prefix %}{{ prefix }}-{% endif %}-{{ palette_name }}: {{ color_value }};";
const COLOR_KEY_TEMPLATE: &str = "{% if prefix %}{{ prefix }}-{% endif %}-{{ palette_name }}-key: {{ palette_key }};";

pub fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  let config_path = cli.config.canonicalize()?;
  let config_dir = config_path.parent().unwrap_or(Path::new("."));
  let config_text = fs::read_to_string(&config_path)?;
  let config: Config = from_str(&config_text)?;

  for theme in config.themes {
    let is_default = theme.default;
    let prefix = theme.prefix;

    for (key, value) in &theme.palettes {
      let base_dyn = parse_color(value.as_str().unwrap()).expect("Invalid color string");
      let base_oklch = base_dyn.convert(color::ColorSpaceTag::Oklch);
      print!("{}", base_oklch);
      let [h, c, l, a] = base_oklch.components;
      let mut tokens = Vec::<String>::new();

      for tone in theme.tones.iter() {
        print!("{}", *tone as f32);
        let variant = OpaqueColor::<Oklch>::new([h, c, *tone as f32]);
        print!("{}\n", variant.to_rgba8())
      }
    }
  }

  Ok(())
}

// fn create_theme_context(theme: &ThemeConfig) -> anyhow::Result<ThemeContext> {
//   let selector = format!(".palette-{}", theme.name);

//   let color_spaces = theme.color_space.iter().map(|cs| {
//     ColorspaceContext {
//       name: cs.clone(),
//       selector: format!("{}.{}", selector, cs)
//     }
//   }).collect();

//   let palettes = theme.palettes.iter().map(|(name, tokens)| {
//     PaletteContext {
//       name: name.clone(),
//       tokens: match tokens.as_object() {
//         Some(map) => map.clone(),
//         None => Map::new()
//       }
//     }
//   }).collect();

//   Ok(ThemeContext {
//     selector,
//     color_spaces,
//     palettes
//   })
// }

// fn main() -> anyhow::Result<()> {
//   let cli = Cli::parse();

//   let config_path = cli.config.canonicalize()?;
//   let config_dir = config_path.parent().unwrap_or(Path::new("."));
//   let config_text = fs::read_to_string(&config_path)?;
//   let config: Config = from_str(&config_text)?;

//   let tera = Tera::new("templates/*.tera")?;

//   for theme in config.themes {
//     let out_dir = normalize_out_dir(config_dir, &config.out_dir);
//     fs::create_dir_all(&out_dir)?;

//     let context = create_context(&theme)?;
//     let theme_out = out_dir.join(format!("{}.css", theme.name));
//     let rendered = tera.render("theme.css.tera", &context)?;
//     fs::write(theme_out, rendered)?;
//   }

//   Ok(())
// }

// fn create_context(theme: &ThemeConfig) -> anyhow::Result<Context> {
//   let mut context = Context::new();
//   context.insert("srgb", "");
//   context.insert("p3", "");
//   context.insert("selector", &format!(".palette-{}", theme.name));
//   context.insert("theme_name", &theme.name);
//   context.insert("description", &theme.description);
//   Ok(context)
// }

// fn create_semantic_context(
//   semantic_name: &str,
//   palette_name: &str,
//   theme: &ThemeConfig,
// ) -> anyhow::Result<Context> {
//   let mut context = Context::new();
//   context.insert("semantic_name", semantic_name);
//   context.insert("palette_name", palette_name);
//   Ok(context)
// }

// fn normalize_out_dir(config_dir: &Path, out: &str) -> PathBuf {
//   let p = Path::new(out);
//   if p.is_absolute() {
//     p.to_path_buf()
//   } else {
//     config_dir.join(p)
//   }
// }