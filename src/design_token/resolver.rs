use indexmap::IndexMap;
use thiserror::Error;

use crate::design_token::token::{Token, TokenSet, TokenValue};

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
  let mut resolved: TokenSet = IndexMap::new();
  let mut stack: Vec<String> = Vec::new();

  for (key, token) in tokens.iter() {
    if resolved.contains_key(key) {
      continue;
    }

    stack.clear();
    stack.push(key.clone());

    let val = resolve_value(key, &token.value, tokens, &mut resolved, &mut stack)
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
) -> Result<TokenValue, ResolveError> {
  match val {
    TokenValue::Reference(target_path) => {
      if !tokens.contains_key(target_path) {
        return Err(ResolveError::TokenNotFound(format!("{} (referenced by {})", target_path, name)));
      }

      let css_var = format!("--{}", target_path.replace('.', "-"));
      Ok(TokenValue::String(format!("var({})", css_var)))
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