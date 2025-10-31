use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Represents the value of a design token, which can be a string, number, boolean,
/// object, alias, reference, or null.
/// - String: A direct string value.
/// - Number: A numeric value.
/// - Bool: A boolean value.
/// - Object: A nested object of token values.
/// - Alias: A reference to another token by its path (used for aliasing).
/// - Reference: A reference to another token by its path (used for CSS variable referencing).
/// - Null: Represents a null value.
/// 
/// This enum is used to define the various types of values that a design token can hold.
/// Aliases are used to create alternative names for existing tokens, while references are used
/// to refer to the value of another token in CSS.
/// For example, an alias might be used to define a "brand color" that points to a specific
/// color token, while a reference would be used to generate a CSS variable that points to
/// that color token's value.
/// 
/// Example:
/// ```
/// let alias_token = Token {
///   name: "color.brand.50".to_string(),
///   value: TokenValue::Alias("color.blue.50".to_string()),
///   comment: Some("Brand color aliasing blue".to_string())
/// };
/// let reference_token = Token {
///   name: "color.primary".to_string(),
///   value: TokenValue::Reference("color.brand.50".to_string()),
///   comment: Some("Primary color referencing brand color".to_string())
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenValue {
  String(String),
  Number(f64),
  Bool(bool),
  Object(IndexMap<String, TokenValue>),
  Alias(String),
  Reference(String),
  Transform(TransformExpr),
  Null
}

/// Represents a design token with a name, value, and optional comment.
/// - name: The name of the token.
/// - value: The value of the token, which can be of various types defined in Token
/// Value.
/// - comment: An optional comment providing additional context about the token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
  pub name: String,
  pub value: TokenValue,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub comment: Option<String>
}

pub type TokenSet = IndexMap<String, Token>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformStep {
  pub r#type: String,
  pub args: Vec<TokenValue>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformExpr {
  pub steps: Vec<TransformStep>
}

#[derive(Debug, Error)]
pub enum ResolveError {
  #[error("alias cycle detected at token: '{0}'")]
  CycleDetected(String),
  #[error("token not found: '{0}'")]
  TokenNotFound(String),
  #[error("type mismatch resolving token: '{0}'")]
  TypeMismatch(String),
  #[error("invalid transform function: '{0}'")]
  InvalidTransform(String),
  #[error("failed to apply transform: '{0}'")]
  TransformFailed(String),
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
      TokenValue::Transform(expr) => apply_transform_pipeline(name, expr, tokens, resolved, stack),
      TokenValue::Reference(target_path) => {
        if !tokens.contains_key(target_path) {
          return Err(ResolveError::TokenNotFound(format!("{} (referenced by {})", target_path, name)));
        }

        let css_var = format!("--{}", target_path.replace('.', "-"));
        Ok(TokenValue::String(format!("var({})", css_var)))
      }
      TokenValue::Alias(target_path) => {
        if stack.contains(&target_path.clone()) {
          return Err(ResolveError::CycleDetected(target_path.clone()));
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

fn apply_transform_pipeline(
  name: &str,
  expr: &TransformExpr,
  tokens: &TokenSet,
  resolved: &mut TokenSet,
  stack: &mut Vec<String>
) -> Result<TokenValue, ResolveError> {
  let mut current: Option<TokenValue> = None;

  for step in &expr.steps {
    current = Some(apply_transform_step(
      name,
      step,
      current.clone(),
      tokens,
      resolved,
      stack
    )?);
  }

  Ok(current.unwrap_or(TokenValue::Null))
}

fn resolve_alias(
  name: &str,
  target: &str,
  tokens: &TokenSet,
  resolved: &mut TokenSet,
  stack: &mut Vec<String>
) -> Result<TokenValue, ResolveError> {
  todo!()
}

fn apply_transform_step(
  name: &str,
  step: &TransformStep,
  input: Option<TokenValue>,
  tokens: &TokenSet,
  resolved: &mut TokenSet,
  stack: &mut Vec<String>
) -> Result<TokenValue, ResolveError> {
  match step.r#type.as_str() {
    "alias" => {
      let target = step
        .args
        .get(0)
        .and_then(|v| match v {
          TokenValue::String(s) => Some(s.clone()),
          _ => None,
        })
        .ok_or_else(|| ResolveError::InvalidTransform("alias requires string arg".into()))?;

      resolve_alias(name, &target, tokens, resolved, stack)
    }

    "multiply" => {
      let factor = match step.args.get(0) {
        Some(TokenValue::Number(n)) => *n,
        _ => return Err(ResolveError::InvalidTransform("multiply expects number".into()))
      };

      match input {
        Some(TokenValue::Number(n)) => Ok(TokenValue::Number(n * factor)),
        _ => Err(ResolveError::TransformFailed("multiply expects number input".into()))
      }
    }

    "lighten" => todo!(),

    other => Err(ResolveError::InvalidTransform(format!("unknown transform: {}", other))),
  }
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
    TokenValue::Reference(r) => format!("reference({})", r),
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
      value: TokenValue::Reference("color.blue.50".to_string()),
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