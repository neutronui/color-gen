use bigcolor::{BigColor, color_space::{OKLCH}};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, from_str};
use tera::{Tera, Context};
use std::{collections::HashMap, fmt::format, fs, ops::Add, path::{Path, PathBuf}};

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
  tones: Option<Vec<u8>>,
  palettes: HashMap<String, PaletteConfig>
}

#[derive(Deserialize, Debug)]
struct PaletteConfig {
  base: String,
  variant: String
}

const DEFAULT_TONE_KEYS: [u8; 11] = [5, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95];
const COLOR_TOKEN_TEMPLATE: &str = "--{% if prefix %}{{ prefix }}-{% endif %}color-{{ palette_name }}-{{ tone }}";
const COLOR_BASE_TEMPLATE: &str = "--{% if prefix %}{{ prefix }}-{% endif %}color-{{ palette_name }}";
const COLOR_KEY_TEMPLATE: &str = "{% if prefix %}{{ prefix }}-{% endif %}color-{{ palette_name }}-key";

pub fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  let config_path = cli.config.canonicalize()?;
  let config_dir = config_path.parent().unwrap_or(Path::new("."));
  let config_text = fs::read_to_string(&config_path)?;
  let config: Config = from_str(&config_text)?;

  let mut tera = Tera::new("templates/*.tera")?;
  tera.add_raw_templates(vec![
    ("COLOR_TOKEN", COLOR_TOKEN_TEMPLATE),
    ("COLOR_BASE", COLOR_BASE_TEMPLATE),
    ("COLOR_KEY", COLOR_KEY_TEMPLATE)
  ])?;

  for theme in config.themes {
    let is_default = theme.default;
    let prefix = theme.prefix;
    let tones = theme.tones.unwrap_or(DEFAULT_TONE_KEYS.to_vec());

    for (key, palette_config) in &theme.palettes {
      let base_color = BigColor::new(&palette_config.base);

      let scale = base_color.monochromatic(Some(tones.len()));
      let mut scale_tokens = HashMap::new();
      
      for (index, color) in scale.iter().enumerate() {
        let tone = tones.get(index).unwrap();
        let mut context = Context::new();
        context.insert("prefix", &prefix);
        context.insert("palette_name", &key);
        context.insert("tone", &format!("{:02}", tone));
        context.insert("color_value", &color.to_hex_string(false));
        let rendered = tera.render("COLOR_TOKEN", &context)?;
        scale_tokens.insert(rendered, color.to_hex_string(false));
      }
    }
  }

  Ok(())
}

fn create_scale_tokens(scale: &Vec<BigColor>, tones: Vec<u8>, prefix: Option<String>) -> anyhow::Result<HashMap<u8, String>> {
  let mut tokens = HashMap::<u8, String>::new();

  for (index, color) in scale.iter().enumerate() {

  }

  Ok(tokens)
}

fn closest_tone(base_color: &BigColor, palette: &Vec<BigColor>) -> anyhow::Result<BigColor> {
  let base_oklch = base_color.to_oklch();

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

fn tonal_palette(base_hex: &str, tones: &[f32]) -> anyhow::Result<Vec<(f32, String)>> {
  let base = BigColor::from_string(base_hex).unwrap();
  let base_oklch = base.to_oklch();

  let mut palette = Vec::with_capacity(tones.len());
  for &tone in tones {
    let l = tone / 100.0;
    let chroma_scale = ((l * (1.0 - l)) * 4.0).powf(0.6);
    let adjusted_c = base_oklch.c * chroma_scale.clamp(0.0, 1.0);

    let tone_oklch = OKLCH {
      l,
      c: adjusted_c,
      h: base_oklch.h,
      alpha: base_oklch.alpha
    };
    let (l, c, h, alpha) = (tone_oklch.l, tone_oklch.c, tone_oklch.h, tone_oklch.alpha);

    let fitted = BigColor::from_oklch(l, c, h, alpha).to_hex_string(false);
    palette.push((tone, fitted));
  }

  Ok(palette)
}

fn create_theme_context() {}

fn create_palette_context() {}

fn create_variant_context() {}