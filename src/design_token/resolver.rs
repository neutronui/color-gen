use indexmap::IndexMap;
use thiserror::Error;

use crate::design_token::token::{Token, TokenSet, TokenValue};
use crate::design_token::css_var::{css_var, CssKeyOptions};

#[derive(Debug, Error)]
pub enum ResolveError {
  #[error("cyclic reference detected at token: '{0}'")]
  CyclicReference(String),
  #[error("token not found: '{0}'")]
  TokenNotFound(String),
  #[error("type mismatch resolving token: '{0}'")]
  TypeMismatch(String),
}

pub fn resolve_tokens(tokens: &TokenSet) -> Result<TokenSet, ResolveError> {
  let opts = CssKeyOptions::default();
  resolve_tokens_with_options(tokens, &opts)
}

pub fn resolve_tokens_with_options(tokens: &TokenSet, opts: &CssKeyOptions) -> Result<TokenSet, ResolveError> {
  let mut resolved: TokenSet = IndexMap::new();
  let mut stack: Vec<String> = Vec::new();

  for (key, token) in tokens.iter() {
    if resolved.contains_key(key) {
      continue;
    }

    stack.clear();
    stack.push(key.clone());

    let val = resolve_value(key, &token.value, tokens, &mut resolved, &mut stack, opts)
      .map_err(|e| match e {
        ResolveError::CyclicReference(s) => ResolveError::CyclicReference(format!(
          "{} -> {}",
          key, s
        )),
        other => other
      })?;

    stack.pop();

    resolved.insert(
      key.clone(),
      Token {
        name: key.clone(),
        value: val,
        comment: token.comment.clone(),
      }
    );
  }

  Ok(resolved)
}

fn resolve_value(
  name: &str,
  val: &TokenValue,
  tokens: &TokenSet,
  resolved: &mut TokenSet,
  stack: &mut Vec<String>,
  opts: &CssKeyOptions,
) -> Result<TokenValue, ResolveError> {
  match val {
    TokenValue::Reference(target_path) => {
      if !tokens.contains_key(target_path) {
        return Err(ResolveError::TokenNotFound(format!("{} (referenced by {})", target_path, name)));
      }

      let key = css_var(target_path, opts);
      Ok(TokenValue::String(key))
    }
    TokenValue::Alias(target_path) => {
      if stack.contains(&target_path.clone()) {
        return Err(ResolveError::CyclicReference(target_path.clone()));
      }

      let target_token = tokens.get(target_path).ok_or_else(|| {
        ResolveError::TokenNotFound(format!("{} (referenced by {})", target_path, name))
      })?;

      if let Some(resolved_token) = resolved.get(target_path) {
        return Ok(resolved_token.value.clone());
      }

      stack.push(target_path.clone());
      let resolved_value = resolve_value(
        target_path,
        &target_token.value,
        tokens,
        resolved,
        stack,
        opts
      )?;
      stack.pop();

      let new_token = Token {
        name: target_path.clone(),
        value: resolved_value.clone(),
        comment: resolved_token_comment(resolved, tokens, target_path)
      };
      resolved.insert(target_path.clone(), new_token);
      Ok(resolved_value)
    }
    TokenValue::Object(map) => {
      let mut new_map = IndexMap::new();
      for (k, v) in map.iter() {
        let rv = resolve_value(name, v, tokens, resolved, stack, opts)?;
        new_map.insert(k.clone(), rv);
      }
      Ok(TokenValue::Object(new_map))
    }
    other => Ok(other.clone()),
  }
}

pub fn merge_token_sets(
  base: &TokenSet,
  overrides: &TokenSet
) -> TokenSet {
  let mut out = base.clone();

  for (key, token) in overrides.iter() {
    out.insert(key.clone(), token.clone());
  }

  out
}

fn resolved_token_comment<'a>(
  resolved: &'a TokenSet,
  tokens: &'a TokenSet,
  key: &str
) -> Option<String> {
  if let Some(t) = resolved.get(key) {
    t.comment.clone()
  } else {
    tokens.get(key).and_then(|t| t.comment.clone())
  }
}

// MARK: - TESTS
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_resolve_tokens() {
    let mut tokens: TokenSet = IndexMap::new();

    tokens.insert(
      "color.primary".to_string(),
      Token {
        name: "color.primary".to_string(),
        value: TokenValue::String("#ff0000".to_string()),
        comment: None,
      }
    );

    tokens.insert(
      "color.secondary".to_string(),
      Token {
        name: "color.secondary".to_string(),
        value: TokenValue::Alias("color.primary".to_string()),
        comment: None,
      }
    );

    tokens.insert(
      "button.background".to_string(),
      Token {
        name: "button.background".to_string(),
        value: TokenValue::Reference("color.secondary".to_string()),
        comment: None,
      }
    );

    let resolved = resolve_tokens(&tokens).unwrap();

    assert_eq!(
      resolved.get("color.primary").unwrap().value,
      TokenValue::String("#ff0000".to_string())
    );

    assert_eq!(
      resolved.get("color.secondary").unwrap().value,
      TokenValue::String("#ff0000".to_string())
    );

    assert_eq!(
      resolved.get("button.background").unwrap().value,
      TokenValue::String(css_var("color.secondary", &CssKeyOptions::default()))
    );
  }

  #[test]
  fn test_cyclic_reference() {
    let mut tokens: TokenSet = IndexMap::new();

    tokens.insert(
      "a".to_string(),
      Token {
        name: "a".to_string(),
        value: TokenValue::Alias("b".to_string()),
        comment: None,
      }
    );

    tokens.insert(
      "b".to_string(),
      Token {
        name: "b".to_string(),
        value: TokenValue::Alias("a".to_string()),
        comment: None,
      }
    );

    let result = resolve_tokens(&tokens);
    assert!(matches!(result, Err(ResolveError::CyclicReference(_))));
  }

  #[test]
  fn test_from_json() {
    let json_data = r##"
    {
      "color.primary": {
        "name": "color.primary",
        "value": "#00ff00"
      },
      "color.accent": {
        "name": "color.accent",
        "value": { "alias": "color.primary" }
      },
      "color.background": {
        "name": "color.background",
        "value": { "reference": "color.primary"  }
      },
      "font": {
        "name": "font",
        "value": {
          "size": "16px",
          "weight": 400,
          "lineHeight": 1.5,
          "color": { "alias": "color.primary" }
        }
      },
      "null": {
        "name": "null",
        "value": null
      },
      "boolean.true": {
        "name": "boolean.true",
        "value": true
      },
      "number.pi": {
        "name": "number.pi",
        "value": 3.14159
      }
    }
    "##;

    let tokens: TokenSet = serde_json::from_str(json_data).unwrap();
    let resolved = resolve_tokens(&tokens).unwrap();

    println!("{:#?}", tokens);

    assert_eq!(
      resolved.get("color.primary").unwrap().value,
      TokenValue::String("#00ff00".to_string())
    );

    assert_eq!(
      resolved.get("color.accent").unwrap().value,
      TokenValue::String("#00ff00".to_string())
    );

    assert_eq!(
      resolved.get("color.background").unwrap().value,
      TokenValue::String(css_var("color.primary", &CssKeyOptions::default()))
    );

    assert_eq!(
      resolved.get("font").unwrap().value,
      TokenValue::Object({
        let mut map = IndexMap::new();
        map.insert("size".to_string(), TokenValue::String("16px".to_string()));
        map.insert("weight".to_string(), TokenValue::Number(400.0));
        map.insert("lineHeight".to_string(), TokenValue::Number(1.5));
        map.insert("color".to_string(), TokenValue::String("#00ff00".to_string()));
        map
      })
    );

    assert_eq!(
      resolved.get("null").unwrap().value,
      TokenValue::Null
    );

    assert_eq!(
      resolved.get("boolean.true").unwrap().value,
      TokenValue::Bool(true)
    );

    assert_eq!(
      resolved.get("number.pi").unwrap().value,
      TokenValue::Number(3.14159)
    );
  }

  #[test]
  fn test_resolve_with_prefix_options() {
    let mut tokens: TokenSet = IndexMap::new();

    tokens.insert(
      "color.primary".to_string(),
      Token { name: "color.primary".to_string(), value: TokenValue::String("#112233".to_string()), comment: None }
    );

    tokens.insert(
      "button.background".to_string(),
      Token { name: "button.background".to_string(), value: TokenValue::Reference("color.primary".to_string()), comment: None }
    );

    let opts = CssKeyOptions { prefix: Some("app".into()), ..Default::default() };
    let resolved = super::resolve_tokens_with_options(&tokens, &opts).unwrap();

    assert_eq!(
      resolved.get("button.background").unwrap().value,
      TokenValue::String(css_var("color.primary", &opts))
    );
  }
}