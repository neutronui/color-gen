#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssKeyOptions {
  pub prefix: Option<String>,
  pub separator: char,
  pub lowercase: bool,
}

impl Default for CssKeyOptions {
  fn default() -> Self {
    Self {
      prefix: None,
      separator: '-',
      lowercase: true,
    }
  }
}

pub fn make_css_custom_property_key(
  path: &str,
  opts: &CssKeyOptions,
) -> String {
  let mut out = String::new();

  if let Some(prefix) = &opts.prefix {
    let norm_prefix = normalize_token(prefix, opts.separator, opts.lowercase);
    if !norm_prefix.is_empty() {
      out.push_str(&norm_prefix);
    }
  }

  let body = path.trim_start().strip_prefix("--").unwrap_or(path.trim());
  let norm_body = normalize_path(body, opts.separator, opts.lowercase);

  if !norm_body.is_empty() {
    if !out.is_empty() {
      out.push(opts.separator);
    }
    out.push_str(&norm_body);
  }

  let mut final_key = String::with_capacity(out.len() + 2);
  final_key.push_str("--");
  final_key.push_str(&out);
  final_key
}

pub fn css_var(path: &str, opts: &CssKeyOptions) -> String {
  format!("var({})", make_css_custom_property_key(path, opts))
}

fn normalize_path(s: &str, separator: char, lowercase: bool) -> String {
  let mut parts = Vec::new();
  let mut current = String::new();

  for ch in s.chars() {
    if ch == '.' || ch == '/' || ch.is_whitespace() {
      if !current.is_empty() {
        parts.push(current.clone());
        current.clear();
      }
    } else {
      current.push(ch);
    }
  }
  if !current.is_empty() {
    parts.push(current);
  }

  let mut out = String::new();
  for (i, token) in parts.iter().map(|t| normalize_token(t, separator, lowercase)).enumerate() {
    if token.is_empty() {
      continue;
    }
    if i > 0 && !out.is_empty() {
      out.push(separator);
    }
    out.push_str(&token);
  }
  out
}

fn normalize_token(s: &str, separator: char, lowercase: bool) -> String {
  let mut out = String::new();
  let mut prev_was_sep = false;
  let mut prev_was_lower_or_digit = false;

  for ch in s.chars() {
    if ch.is_ascii_alphanumeric() {
      if ch.is_ascii_uppercase() {
        if prev_was_lower_or_digit && !prev_was_sep {
          if !out.ends_with(separator) {
            out.push(separator);
          }
        }
        let lc = ch.to_ascii_lowercase();
        out.push(if lowercase { lc } else { ch });
        prev_was_sep = false;
        prev_was_lower_or_digit = ch.is_ascii_alphabetic() || ch.is_ascii_digit();
      } else {
        let c = if lowercase { ch.to_ascii_lowercase() } else { ch };
        out.push(c);
        prev_was_sep = false;
        prev_was_lower_or_digit = ch.is_ascii_alphabetic() || ch.is_ascii_digit();
      }
    } else {
      if !out.ends_with(separator) {
        out.push(separator);
      }
      prev_was_sep = true;
      prev_was_lower_or_digit = false;
    }
  }

  let trimmed = out.trim_matches(separator).to_string();
  trimmed
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_simple_dot_separated() {
    let opts = CssKeyOptions::default();
    let key = make_css_custom_property_key("color.primary.background", &opts);
    assert_eq!(key, "--color-primary-background");
  }

  #[test]
    fn default_slash_and_spaces() {
      let opts = CssKeyOptions::default();
      assert_eq!(
        make_css_custom_property_key("color/theme / primary 500", &opts),
        "--color-theme-primary-500"
      );
    }

    #[test]
    fn preserves_leading_dashes_once() {
      let opts = CssKeyOptions::default();
      assert_eq!(
        make_css_custom_property_key("--color.primary-500", &opts),
        "--color-primary-500"
      );
    }

    #[test]
    fn camel_and_pascal_case_boundaries() {
      let opts = CssKeyOptions::default();
      assert_eq!(
        make_css_custom_property_key("Color.PrimaryAccent", &opts),
        "--color-primary-accent"
      );
      assert_eq!(
        make_css_custom_property_key("borderRadius.sm", &opts),
        "--border-radius-sm"
      );
    }

    #[test]
    fn custom_separator_underscore() {
      let opts = CssKeyOptions {
        prefix: None,
        separator: '_',
        lowercase: true,
      };
      assert_eq!(
        make_css_custom_property_key("color.primary.500", &opts),
        "--color_primary_500"
      );
    }

    #[test]
    fn with_prefix_namespace() {
      let opts = CssKeyOptions {
        prefix: Some("dark".to_string()),
        ..Default::default()
      };
      assert_eq!(
        make_css_custom_property_key("color.primary.500", &opts),
        "--dark-color-primary-500"
      );
    }

    #[test]
    fn complex_input_sanitization_and_collapse() {
      let opts = CssKeyOptions::default();
      assert_eq!(
        make_css_custom_property_key("layout..grid   cols", &opts),
        "--layout-grid-cols"
      );
      assert_eq!(
        make_css_custom_property_key("color---primary", &opts),
        "--color-primary"
      );
      assert_eq!(
        make_css_custom_property_key("size(2x)@md", &opts),
        "--size-2x-md"
      );
    }

    #[test]
    fn css_var_wrapper() {
      let opts = CssKeyOptions {
        prefix: Some("theme".to_string()),
        ..Default::default()
      };
      assert_eq!(
        css_var("Color.Primary.500", &opts),
        "var(--theme-color-primary-500)"
      );
    }

    #[test]
    fn css_var_default_reference_style() {
      let opts = CssKeyOptions::default();
      assert_eq!(css_var("a.b.c", &opts), "var(--a-b-c)");
      assert_eq!(css_var("button/Primary.sizeLg", &opts), "var(--button-primary-size-lg)");
    }

    #[test]
    fn css_var_with_prefix_reference_style() {
      let opts = CssKeyOptions { prefix: Some("app".into()), ..Default::default() };
      assert_eq!(css_var("a.b.c", &opts), "var(--app-a-b-c)");
      assert_eq!(css_var("Color.Primary", &opts), "var(--app-color-primary)");
    }

    #[test]
    fn empty_like_inputs_do_not_break() {
      let opts = CssKeyOptions::default();
      assert_eq!(make_css_custom_property_key("", &opts), "--");
      assert_eq!(make_css_custom_property_key("--", &opts), "--");
      assert_eq!(make_css_custom_property_key("   ", &opts), "--");
    }

    #[test]
    fn prefix_is_normalized() {
      let opts = CssKeyOptions {
        prefix: Some("Dark Mode".into()),
        ..Default::default()
      };
      assert_eq!(
        make_css_custom_property_key("Color.Primary", &opts),
        "--dark-mode-color-primary"
      );
    }

    #[test]
    fn mixed_separators_and_camelcase() {
      let opts = CssKeyOptions::default();
      assert_eq!(
        make_css_custom_property_key("button/PrimaryLabel.sizeLg", &opts),
        "--button-primary-label-size-lg"
      );
    }
}