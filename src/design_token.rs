use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenValue {
  String(String),
  Number(f64),
  Bool(bool),
  Object(IndexMap<String, TokenValue>),
  Alias(String),
  Null
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
  pub name: String,
  pub value: TokenValue,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub comment: Option<String>
}

pub type TokenSet = IndexMap<String, Token>;

#[derive(Debug, Error)]
pub enum ResolveError {
  #[error("alias cycle detected at token: '{0}'")]
  CycleDetected(String),
  #[error("token not found: '{0}'")]
  TokenNotFound(String),
  #[error("type mismatch resolving token: '{0}'")]
  TypeMismatch(String)
}

pub fn resolve_tokens(tokens: &TokenSet) -> Result<TokenSet, ResolveError> {
  let mut resolved: TokenSet = IndexMap::new();
  let mut stack: Vec<String> = Vec::new();

  fn resolve_value(
    name: &str,
    val: &TokenValue,
    tokens: &TokenSet,
    resolved: &mut TokenSet,
    stack: &mut Vec<String>
  ) -> Result<TokenValue, ResolveError> {
    match val {
      TokenValue::Alias(target_path) => {
        if stack.contains(&target_path.clone()) {
          return Err(ResolveError::CycleDetected(target_path.clone()));
        }

        let target_token = tokens.get(target_path).ok_or_else(|| {
          ResolveError::TokenNotFound(format!("{} (referencedby {})", target_path, name))
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
          stack
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
          let rv = resolve_value(name, v, tokens, resolved, stack)?;
          new_map.insert(k.clone(), rv);
        }
        Ok(TokenValue::Object(new_map))
      }
      other => Ok(other.clone()),
    }
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

  for (key, token) in tokens.iter() {
    if resolved.contains_key(key) {
      continue;
    }

    stack.clear();
    stack.push(key.clone());

    let val = resolve_value(key, &token.value, tokens, &mut resolved, &mut stack)
      .map_err(|e| match e {
        ResolveError::CycleDetected(s) => ResolveError::CycleDetected(format!(
          "{} -> {}",
          key, s
        )),
        other => other,
      })?;

    stack.pop();

    resolved.insert(
      key.clone(),
      Token {
        name: key.clone(),
        value: val,
        comment: token.comment.clone(),
      },
    );
  }

  Ok(resolved)
}

pub fn to_css_custom_properties(tokens: &TokenSet) -> IndexMap<String, String> {
  let mut map = IndexMap::new();
  for (key, token) in tokens.iter() {
    let css_name = format!("--{}", key.replace('.', "-"));
    let css_value = token_value_to_string(&token.value);

    map.insert(css_name, css_value);
  }

  map
}

fn token_value_to_string(value: &TokenValue) -> String {
  match value {
    TokenValue::String(s) => s.clone(),
    TokenValue::Number(n) => {
      if (n.fract()).abs() < std::f64::EPSILON {
        format!("{:.0}", n)
      } else {
        n.to_string()
      }
    },
    TokenValue::Bool(b) => b.to_string(),
    TokenValue::Object(obj) => {
      serde_json::to_string(obj).unwrap_or_else(|_| String::from("{}"))
    },
    TokenValue::Alias(a) => format!("alias({})", a),
    TokenValue::Null => String::from("null")
  }
}

pub fn merge_token_sets(base: &TokenSet, overrides: &TokenSet) -> TokenSet {
  let mut out = base.clone();
  for (k, v) in overrides.iter() {
    out.insert(k.clone(), v.clone());
  }
  out
}

pub fn example() -> Result<(), ResolveError> {
  let mut tokens: TokenSet = IndexMap::new();

  tokens.insert(
    "color.blue.50".to_string(),
    Token {
      name: "color.blue.50".to_string(),
      value: TokenValue::String("#3500ff".to_string()),
      comment: Some("A bright blue color".to_string())
    },
  );

  tokens.insert(
    "color.brand.50".to_string(),
    Token {
      name: "color.brand.50".to_string(),
      value: TokenValue::Alias("color.blue.50".to_string()),
      comment: Some("Brand color aliasing blue".to_string())
    },
  );

  let resolved = resolve_tokens(&tokens)?;
  let css_map = to_css_custom_properties(&resolved);

  for (name, value) in css_map.iter() {
    println!("{}: {};", name, value);
  }

  Ok(())
}