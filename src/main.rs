use std::{collections::{BTreeMap, HashMap}, fs, path::{Path, PathBuf}};
use indexmap::IndexMap;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use clap::Parser;
use bigcolor::BigColor;
use serde_json::from_str;
use tera::Tera;

mod design_token;

const VARIANT_TEMPLATE: &str = r#"
{% for variant in variants -%}
{{ variant.selector }} {
  {%- for variable in variant.variables %}
  {{ variable }}
  {%- endfor %}
}
{% endfor %}
"#;

const PALETTE_TEMPLATE: &str = r#"
/*
  {{ name }} Palette
  {{ description }}
*/

{% for variant_file in variant_file_names -%}
@import("./{{ variant_file }}");
{% endfor -%}
{{ selector }} {
  {%- for variable in variables %}
  {{ variable }}
  {%- endfor %}
}
"#;

lazy_static! {
  pub static ref TEMPLATES: Tera = {
    let mut tera = Tera::default();

    tera.add_raw_templates(vec![
      ("palette.css.tera", PALETTE_TEMPLATE),
      ("variant.css.tera", VARIANT_TEMPLATE)
    ]).expect("Could not add templates");

    tera
  };
}

#[derive(Debug, Parser)]
#[command(name = "substrate-color-gen")]
#[command(bin_name = "substrate")]
#[command(about, version)]
struct Cli {
  #[arg(short, long, value_name = "FILE_PATH")]
  pub config: PathBuf,

  /// Optional path to an input JSON file containing a token set to resolve.
  /// If provided, the tool will resolve tokens and emit CSS/JSON outputs.
  #[arg(long, value_name = "TOKENS_JSON")] 
  pub tokens: Option<PathBuf>,

  /// Output directory for resolved token outputs (CSS and JSON). Used only when --tokens is provided.
  #[arg(long, value_name = "OUT_DIR")] 
  pub tokens_out_dir: Option<PathBuf>,

  /// Optional CSS selector used in the generated stylesheet (default :root).
  #[arg(long, default_value = ":root")]
  pub tokens_selector: String,

  /// Optional prefix for CSS custom properties when generating the stylesheet.
  #[arg(long, default_value = "")]
  pub tokens_prefix: String,

  /// Load one or more JS plugin files providing custom transforms (requires --features js).
  #[arg(long = "tokens-plugin", value_name = "JS_FILE", num_args(0..))]
  pub tokens_plugins: Vec<PathBuf>,

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

#[derive(Debug, Serialize)]
struct CssVariant {
  selector: String,
  variables: Vec<String>
}

#[derive(Debug, Serialize)]
struct CssVariantCtx {
  variants: Vec<CssVariant>
}

#[derive(Debug, Serialize)]
struct CssPaletteCtx {
  name: String,
  description: String,
  variant_file_names: Vec<String>,
  selector: String,
  variables: Vec<String>
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

trait StringExtensions {
  fn with_prefix(&self, prefix: &str) -> String;
}

impl StringExtensions for String {
  fn with_prefix(&self, prefix: &str) -> String {
    if prefix.is_empty() {
      self.clone()
    } else {
      format!("{}-{}", prefix, self)
    }
  }
}

fn main() {
  let cli = Cli::parse();
  // If token I/O is requested, run that path and return early
  if let Some(tokens_path) = &cli.tokens {
    let tokens_json = fs::read_to_string(tokens_path).expect("Failed to read tokens JSON");
    let raw_map: IndexMap<String, design_token::Token> = serde_json::from_str(&tokens_json)
      .expect("Failed to parse tokens JSON into TokenSet (map)");
    let mut token_set: design_token::TokenSet = IndexMap::new();
    for (k, t) in raw_map {
      token_set.insert(k, t);
    }

    // Build registry and optionally load JS plugins
  #[allow(unused_mut)]
  let mut registry = design_token::TransformRegistry::default();
    #[cfg(feature = "js")]
    {
      for p in &cli.tokens_plugins {
        let src = fs::read_to_string(p).expect("Failed to read JS plugin file");
        // Use file stem as a default transform group name prefix; allow multiple functions inside.
        // We register the whole file content to be evaluated together later.
        registry.add_js_transform(
          p.file_stem().and_then(|s| s.to_str()).unwrap_or("plugin"),
          &src,
        );
      }
    }

    let resolved = design_token::resolve_tokens_with_registry(&token_set, &registry)
      .expect("Failed to resolve tokens");
    let out_dir = cli.tokens_out_dir.clone().unwrap_or_else(|| PathBuf::from("out"));
    fs::create_dir_all(&out_dir).expect("Failed to create tokens output directory");

    // Write CSS
    let css = design_token::to_css_stylesheet(&resolved, &cli.tokens_selector, 
      if cli.tokens_prefix.is_empty() { None } else { Some(cli.tokens_prefix.as_str()) }
    );
    let css_path = out_dir.join("tokens.css");
    fs::write(&css_path, css).expect("Failed to write tokens.css");

    // Write resolved JSON (string values for convenience)
    let resolved_map = design_token::to_resolved_string_map(&resolved);
    let json_path = out_dir.join("tokens.resolved.json");
    fs::write(&json_path, serde_json::to_string_pretty(&resolved_map).unwrap())
      .expect("Failed to write tokens.resolved.json");

    println!("Resolved tokens written to {:?} and {:?}", css_path, json_path);
    return;
  }

  let data = fs::read_to_string(&cli.config)
    .expect("Failed to read config file");
  let config: Config = from_str(&data)
    .expect("Failed to parse config JSON");
  let out_dir = normalize_out_dir(&cli.config.parent().unwrap_or(Path::new(".")), &config.out_dir);
  fs::create_dir_all(&out_dir).expect("Failed to create output directory");
  
  println!("Output directory: {:?}", out_dir);

  for palette_cfg in &config.palettes {
    let name = &palette_cfg.name;
    let description = palette_cfg.description.as_deref().unwrap_or_default();
    let is_default = palette_cfg.default.unwrap_or_default();
    let prefix = palette_cfg.prefix.as_deref().unwrap_or_default();
    let tones = &palette_cfg.tones;
    let palette_out_dir = out_dir.join(name);
    fs::create_dir_all(&palette_out_dir).expect("Failed to create palette output directory");
    
    let mut scales = Vec::<Scale>::new();
    let default_selector = ":where(:root)";
    
    for (hue_name, base_color_raw) in &palette_cfg.hues {
      let mut shades = BTreeMap::<u8, Shade>::new();
      let base_color = BigColor::new(base_color_raw);
      let colors = base_color.monochromatic(Some(tones.len()));

      for (idx, color) in colors.iter().enumerate() {
        shades.insert(tones[idx], Shade {
          tone: tones[idx],
          // TODO: Expose adjustments via config or CLI
          color: color.clone().lighten(Some(5.0)).clone(),
          name: format!("color-{}-{:02}", hue_name, tones[idx]).with_prefix(prefix)
        });
      }

      let key_tone = closest_to_base(&base_color, &shades);

      scales.push(Scale {
        name: hue_name.clone(),
        shades,
        key_tone
      });
    }

    let mut variant_file_names = Vec::<String>::new();

    for (variant, hue) in &palette_cfg.variants {
      let variant_file_name = format!("{}.css", variant);
      let out_path = palette_out_dir.join(&variant_file_name);
      variant_file_names.push(variant_file_name.clone());

      let mut variants = Vec::<CssVariant>::new();
      
      let mut ordered_scales: Vec<&Scale> = Vec::with_capacity(scales.len());
      if let Some(pos) = scales.iter().position(|s| s.name == *hue) {
        ordered_scales.push(&scales[pos]);
        for (i, s) in scales.iter().enumerate() {
          if i != pos {
            ordered_scales.push(s);
          }
        }
      } else {
        for s in &scales {
          ordered_scales.push(s);
        }
      }

      for scale in ordered_scales {
        let selector = if scale.name == *hue {
          format!("{}, .{}-{}", default_selector, variant.with_prefix(prefix), scale.name)
        } else {
          format!(".{}-{}", variant.with_prefix(prefix), scale.name)
        };

        let mut css_vars = Vec::<String>::new();

        for shade in scale.shades.values() {
          let var_name = format!("color-{}-{:02}", variant, shade.tone).with_prefix(prefix);
          css_vars.push(format!("{}: var(--{});", format!("--{}", var_name), shade.name));
        }
        let key_color_prop = format!("color-{}", scale.name).with_prefix(prefix);
        let key_color_name = scale.shades.get(&scale.key_tone).unwrap().name.clone();
        css_vars.push(format!("--{}: var(--{});", key_color_prop, key_color_name));
        let key_on_prop = format!("color-{}-on", scale.name).with_prefix(prefix);
        let key_on_name = format!("color-{}-on", scale.name).with_prefix(prefix);
        css_vars.push(format!("--{}: var(--{});", key_on_prop, key_on_name));

        variants.push(CssVariant {
          selector,
          variables: css_vars
        });
      }

      let ctx = tera::Context::from_serialize(CssVariantCtx {
        variants
      }).unwrap();
      
      println!("Generated variant file: {:?}", out_path);
      let rendered = TEMPLATES.render("variant.css.tera", &ctx).unwrap();
      fs::write(&out_path, rendered).expect("Failed to write output file");
    }

    let palette_file_name = format!("{}.css", &palette_cfg.name);
    let selector = if is_default {
          format!("{}, .{}-palette", default_selector, &palette_cfg.name.with_prefix(prefix))
        } else {
          format!(".{}-palette", &palette_cfg.name.with_prefix(prefix))
        };
    let mut css_vars = Vec::<String>::new();
    
    for scale in &scales {
      for shade in scale.shades.values() {
        css_vars.push(format!("--{}: {};", shade.name, shade.color.to_oklch_string()));
      }

      let key_color_prop = format!("color-{}", scale.name).with_prefix(prefix);
      let key_color_name = scale.shades.get(&scale.key_tone).unwrap().name.clone();
      let key_prop = format!("{}-key", scale.name).with_prefix(prefix);

      css_vars.push(format!("--{}: var(--{});", key_color_prop, key_color_name));
      css_vars.push(format!("--{}: {};", key_prop, scale.key_tone));
      if !std::ptr::eq(scale, scales.last().unwrap()) {
        css_vars.push(String::new());
      }
    }

    let ctx = tera::Context::from_serialize(CssPaletteCtx {
      name: name.clone(),
      description: description.to_string(),
      variant_file_names,
      selector,
      variables: css_vars
    }).unwrap();

    let out_path = palette_out_dir.join(&palette_file_name);
    println!("Generated palette file: {:?}", out_path);
    let rendered = TEMPLATES.render("palette.css.tera", &ctx).unwrap();
    fs::write(&out_path, rendered).expect("Failed to write output file");

    println!("================================================");
    design_token::example().expect("Design token example failed");
  }
}