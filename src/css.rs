use std::collections::HashMap;

use bigcolor::BigColor;

use crate::{config::PaletteConfig, TEMPLATES};

fn tonal_steps() -> [u8; 11] {
  [05, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95]
}

enum TokenType {
  Color,
  Base,
  Key
}

impl TokenType {
  fn as_str(&self) -> &str {
    match self {
      TokenType::Color => "COLOR_TOKEN",
      TokenType::Base => "COLOR_BASE",
      TokenType::Key => "COLOR_KEY",
    }
  }
}

enum TokenState {
  Uninitialized,
  WithoutValue(String),
  WithValue,
  Rendered(String)
}

pub trait TokenStateBehavior: Sized {
  
}

impl TokenStateBehavior for TokenState {
  
}



pub struct CSSColorToken {
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

  pub fn to_string(&self, with_value: bool) -> String {
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

pub struct CSSBaseToken {
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

  pub fn to_string(&self) -> String {
    let mut context = tera::Context::new();
    context.insert("prefix", &self.prefix);
    context.insert("palette_name", &self.palette_name);

    let mut token = TEMPLATES.render("COLOR_BASE", &context).unwrap();
    token = format!("{}: {};", token, self.value);

    token
  }
}

pub struct CSSKeyToken {
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

  pub fn to_string(&self) -> String {
    let mut context = tera::Context::new();
    context.insert("prefix", &self.prefix);
    context.insert("palette_name", &self.palette_name);

    let mut token = TEMPLATES.render("COLOR_KEY", &context).unwrap();
    token = format!("{}: {};", token, format!("{:02}", self.value));

    token
  }
}

pub struct Palette {
  pub name: String,
  pub for_variant: Option<String>,
  pub tokens: HashMap<u8, CSSColorToken>,
  pub key: CSSKeyToken,
  pub base: CSSBaseToken
}

pub fn generate_palette_css(name: &str, config: &PaletteConfig) -> Result<Palette, Box<dyn std::error::Error>> {
  let source_color = BigColor::new(&config.base);
  let source_scale = source_color.monochromatic(Some(tonal_steps().len()));
  let mut color_tokens: HashMap<u8, CSSColorToken> = HashMap::new();
  let key_color = closest_to_base(&source_color, &source_scale)?;
  let key_tone = source_scale.iter().position(|c| c == &key_color).unwrap() as u8;

  for (index, color) in source_scale.iter().enumerate() {
    let tone = tonal_steps()[index];
    let token = CSSColorToken::new(
      None,
      name.to_string(),
      tone,
      color.clone()
    );

    color_tokens.insert(tone, token);
  }

  let key_token = CSSKeyToken::new(
    None,
    name.to_string(),
    key_tone as u8
  );

  let base_token = CSSBaseToken::new(
    None,
    name.to_string(),
    color_tokens.get(&key_tone).unwrap().to_string(false)
  );

  Ok(Palette {
    name: name.to_string(),
    for_variant: config.variant.clone(),
    tokens: color_tokens,
    key: key_token,
    base: base_token
  })
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