// use std::{collections::{BTreeMap, HashMap}, fs, path::{Path, PathBuf}};
// use bigcolor::BigColor;
// use clap::Parser;
// use lazy_static::lazy_static;
// use serde::{Deserialize, Serialize};
// use serde_json::from_str;
// use tera::Tera;

// lazy_static! {
//   static ref TEMPLATES: Tera = {
//     let tera = match Tera::new("templates/*.tera") {
//       Ok(t) => t,
//       Err(e) => {
//         println!("Parsing error(s): {}", e);
//         std::process::exit(1);
//       }
//     };

//     tera
//   };
// }

// #[derive(Parser, Debug)]
// #[command(version, about, long_about = None)]
// pub struct Cli {
//   #[arg(short, long, value_name = "FILE_PATH")]
//   pub config: PathBuf,

//   #[arg(short, long, action = clap::ArgAction::Count)]
//   debug: u8,
// }


// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Config {
//   pub out_dir: String,
//   pub themes: Vec<ThemeConfig>
// }

// #[derive(Debug, Deserialize)]
// pub struct ThemeConfig {
//   pub name: String,
//   pub default: Option<bool>,
//   pub description: Option<String>,
//   pub prefix: Option<String>,
//   pub palettes: HashMap<String, String>,
//   pub variants: HashMap<String, String>
// }

// #[derive(Debug)]
// pub struct Theme {
//   pub name: String,
//   pub default: bool,
//   pub palettes: Vec<Palette>,
//   pub variants: Vec<Variant>
// }

// #[derive(Debug)]
// pub struct Palette {
//   pub name: String,
//   pub colors: BTreeMap<u8, BigColor>,
//   pub base_color: BigColor,
//   pub key_tone: u8
// }

// #[derive(Debug)]
// pub struct Variant {
//   pub name: String,
//   pub palette: String,
// }

// #[derive(Debug, Serialize)]
// pub struct CssTheme {
//   default: bool,
//   name: String,
//   palettes: Vec<CssPalette>,
// }

// #[derive(Debug, Serialize)]
// pub struct CssPalette {
//   name: String,
//   colors: BTreeMap<String, String>,
//   base_color: (String, String),
//   key_tone: u8
// }

// pub struct CssValue {}

// pub enum CssGenState {
//   Raw(RawConfig),
//   Validated(ValidatedConfig),
//   Colors(GeneratedColors),
//   Tokens(GeneratedTokens),
//   Css(CssReady),
// }

// pub struct RawConfig(pub Config);
// pub struct ValidatedConfig(pub Config);
// pub struct GeneratedColors(pub Vec<Theme>);
// pub struct GeneratedTokens(pub Vec<CssTheme>);
// pub struct CssReady(pub Vec<Theme>);

// pub trait Validate {
//   fn validate(self) -> ValidatedConfig;
// }

// pub trait GenerateColors {
//   fn generate_colors(&self) -> GeneratedColors;
//   fn get_closest_to_base(&self, base_color: &BigColor, scale: &Vec<BigColor>) -> Result<BigColor, Box<dyn std::error::Error>>;
// }

// pub trait GenerateTokens {
//   fn generate_tokens(self) -> GeneratedTokens;
// }

// pub trait EmitCss {
//   fn emit_css(self, out_dir: &str) -> CssReady;
// }

// impl Validate for RawConfig {
//   fn validate(self) -> ValidatedConfig {
//     ValidatedConfig(self.0)
//   }
// }

// impl GenerateColors for ValidatedConfig {
//   fn generate_colors(&self) -> GeneratedColors {
//     let mut themes = Vec::new();
//     let tones = vec![05, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95];

//     for theme_cfg in &self.0.themes {
//       let mut palettes = Vec::new();
//       let mut variants = Vec::new();

//       for (name, base_color_raw) in &theme_cfg.palettes {
//         let base_color = BigColor::new(base_color_raw);
//         let scale = base_color.monochromatic(Some(tones.len()));
//         let key_color = self.get_closest_to_base(&base_color, &scale).unwrap();
//         let colors: BTreeMap<u8, BigColor> = tones
//           .iter()
//           .cloned()
//           .zip(scale.into_iter())
//           .collect();

//         palettes.push(Palette {
//           name: name.clone(),
//           colors: colors.clone(),
//           base_color: base_color.clone(),
//           key_tone: colors.iter().position(|c| c.1 == &key_color).unwrap() as u8,
//         });
//       }

//       for (name, palette_name) in &theme_cfg.variants {
//         variants.push(Variant {
//           name: name.clone(),
//           palette: palette_name.clone(),
//         });
//       }

//       themes.push(Theme {
//         name: theme_cfg.name.clone(),
//         default: theme_cfg.default.unwrap_or(false),
//         palettes,
//         variants
//       });
//     }


//     GeneratedColors(themes)
//   }

//   fn get_closest_to_base(&self, base_color: &BigColor, scale: &Vec<BigColor>) -> Result<BigColor, Box<dyn std::error::Error>> {
//     let base_oklch = base_color.to_oklch();
//     let closest = scale
//       .iter()
//       .min_by(|a, b| {
//         (a.to_oklch().l - base_oklch.l)
//           .abs()
//           .partial_cmp(&(b.to_oklch().l - base_oklch.l).abs())
//           .unwrap()
//       })
//       .unwrap_or(scale.get(scale.len() / 2).unwrap());

//     Ok(closest.clone())
//   }
// }

// impl GenerateTokens for GeneratedColors {
//   fn generate_tokens(self) -> GeneratedTokens {
//     let mut themes = Vec::new();
//     let tones = vec![05, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95];

//     for theme in &self.0 {
//       let mut css_palettes = Vec::new();

//       for palette in &theme.palettes {
//         let mut colors_map: BTreeMap<String, String> = BTreeMap::new();

//         for (tone, color) in &palette.colors {
//           colors_map.insert(format!("{:02}", tone), color.to_oklch_string());
//         }

//         let mut base_color_ctx = tera::Context::new();
//         base_color_ctx.insert("palette_name", &palette.name);

//         css_palettes.push(CssPalette {
//           name: palette.name.clone(),
//           colors: colors_map,
//           base_color: (
//             TEMPLATES.render("color_base.css.tera", &base_color_ctx).unwrap().to_string(),
//             palette.base_color.to_oklch_string()
//           ),
//           key_tone: palette.key_tone
//         });
//       }

//       themes.push(CssTheme {
//         name: theme.name.clone(),
//         default: theme.default,
//         palettes: css_palettes,
//       });
//     }

//     GeneratedTokens(themes)
//   }
// }

// impl EmitCss for GeneratedTokens {
//   fn emit_css(self, out_dir: &str) -> CssReady {
//     let themes = Vec::new();
//     for theme in &self.0 {
//       let mut context = tera::Context::new();
//       context.insert("name", &theme.name);
//       context.insert("default", &theme.default);
//       context.insert("palettes", &theme.palettes);

//       let rendered = TEMPLATES
//         .render("palette.css.tera", &context)
//         .expect("Failed to render template");

//       let theme_out_dir = Path::new(out_dir).join(&theme.name);
//       fs::create_dir_all(&theme_out_dir).expect("Failed to create theme output directory");
//       let out_path = theme_out_dir.join("palette.css");
//       fs::write(&out_path, rendered).expect("Failed to write CSS file");
//     }

//     CssReady(themes)
//   }
// }

// fn normalize_out_dir(config_dir: &Path, out: &str) -> PathBuf {
//   let p = Path::new(out);
//   if p.is_absolute() {
//     p.to_path_buf()
//   } else {
//     config_dir.join(p)
//   }
// }

// fn main() {
//   let cli = Cli::parse();
//   let data = fs::read_to_string(&cli.config).unwrap();
//   let config: Config = from_str(&data).unwrap();
//   let raw: RawConfig = RawConfig(config);
//   let config_dir = &cli.config.parent().unwrap_or(Path::new("."));
//   let out_dir = normalize_out_dir(config_dir, &raw.0.out_dir);
//   fs::create_dir_all(&out_dir).expect("Failed to create output directory");

  
//   let validated = raw.validate();
//   let colors = validated.generate_colors();
//   let tokens = colors.generate_tokens();
//   let css_ready = tokens.emit_css(&out_dir.to_str().unwrap());
// }

use std::collections::{HashMap, HashSet};

use bigcolor::BigColor;
use serde::Deserialize;

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
struct Shade {
  tone: u8,
  color: BigColor
}

#[derive(Debug, Clone)]
struct Hue {
  name: String,
  base: String,
  shades: Vec<Shade>
}

#[derive(Debug, Clone)]
struct Variant {
  name: String,
  hue_ref: String
}

#[derive(Debug, Clone)]
struct Palette {
  name: String,
  description: Option<String>,
  prefix: Option<String>,
  hues: HashMap<String, Hue>,
  variants: HashMap<String, Variant>
}

fn generate_tonal_shades(base_color_raw: &str, tones: &[u8]) -> Vec<Shade> {
  let base_color = BigColor::new(base_color_raw);
  let scale = base_color.monochromatic(Some(tones.len()));
  scale.iter().enumerate().map(|(idx, color)| {
    Shade {
      tone: tones[idx],
      color: color.clone()
    }
  })
  .collect()
}

fn generate_palette(config: &PaletteConfig) -> Palette {
  let hues = config.hues.iter()
    .map(|(name, base_raw)| {
      let shades = generate_tonal_shades(base_raw, &config.tones);
      let hue = Hue {
        name: name.clone(),
        base: base_raw.clone(),
        shades
      };
      (name.clone(), hue)
    })
    .collect::<HashMap<_, _>>();

  let variants = config.variants.iter()
    .map(|(name, hue_ref)| {
      let variant= Variant {
        name: name.clone(),
        hue_ref: hue_ref.clone()
      };
      (name.clone(), variant)
    })
    .collect::<HashMap<_, _>>();

  Palette {
    name: config.name.clone(),
    description: config.description.clone(),
    prefix: config.prefix.clone(),
    hues,
    variants
  }
}

#[derive(Debug, Clone)]
struct CssVarNode {
  name: String,
  value: String
}

fn build_dependency_graph(palette: &Palette) -> Vec<CssVarNode> {
  let mut nodes = Vec::new();

  for (hue_name, hue) in &palette.hues {
    for shade in &hue.shades {
      let var_name = format!("--{}-{:02}", hue_name, shade.tone);
      nodes.push(CssVarNode {
        name: var_name.clone(),
        value: shade.clone().color.lighten(Some(5.0)).to_oklch_string()
      });
    }
  }

  for (variant_name, variant) in &palette.variants {
    for tone in &[50u8] {
      let ref_name = format!("--{}-{:02}", variant.hue_ref, tone);
      let var_name = format!("--{}-{:02}", variant_name, tone);
      nodes.push(CssVarNode {
        name: var_name.clone(),
        value: format!("var({})", ref_name),
      });
    }
  }

  nodes
}

fn generate_css(nodes: &[CssVarNode]) -> String {
  let mut css = String::from(":root {\n");
  for node in nodes {
    css.push_str(&format!("  {}: {};\n", node.name, node.value));
  }
  css.push_str("}\n");
  css
}

fn main() {
    let json = r##"
    {
        "name": "test",
        "default": false,
        "description": "test theme",
        "prefix": null,
        "tones": [5,10,20,30,40,50,60,70,80,90,95],
        "hues": {
            "blue": "#3500FF",
            "gray": "#1C1C20",
            "red": "#FF3B30"
        },
        "variants": {
            "brand": "blue",
            "neutral": "gray",
            "danger": "red"
        }
    }
    "##;

    let config: PaletteConfig = serde_json::from_str(json).unwrap();
    let palette = generate_palette(&config);
    let nodes = build_dependency_graph(&palette);
    let css = generate_css(&nodes);

    println!("{}", css);
}