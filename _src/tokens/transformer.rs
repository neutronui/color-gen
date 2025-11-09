use crate::config::{Config, Transform, TransformTarget};

pub fn transform_tokens(config: Config) {
  for transform in config.transforms {
    let from_path = transform.from;
    let targets = transform.to;

    // Load tokens from the 'from' path
    let tokens = load_tokens(&from_path);

    for target in targets {
      match target.format {
        crate::config::TargetFormat::Json => {
          save_as_json(&tokens, &target.output);
        },
        crate::config::TargetFormat::Toml => {
          save_as_toml(&tokens, &target.output);
        },
        crate::config::TargetFormat::Yaml => {
          save_as_yaml(&tokens, &target.output);
        },
        crate::config::TargetFormat::Scss => {
          save_as_scss(&tokens, &target.output);
        },
        crate::config::TargetFormat::Css => {
          save_as_css(&tokens, &target.output);
        },
        crate::config::TargetFormat::Mjs => {
          save_as_mjs(&tokens, &target.output);
        },
      }
    }
  }
}