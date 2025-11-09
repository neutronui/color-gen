use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TokenJSON {
  #[serde(rename = "$type", skip_serializing_if = "Option::is_none")]
  pub token_type: Option<String>,
  #[serde(rename = "$description", skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename = "$value", skip_serializing_if = "Option::is_none")]
  pub value: Option<Value>,
  #[serde(rename = "$deprecated", skip_serializing_if = "Option::is_none")]
  pub deprecated: Option<bool>,
  // Dedicated nested object for extensions under "$extensions"
  #[serde(rename = "$extensions", skip_serializing_if = "Option::is_none")]
  pub extensions: Option<IndexMap<String, Value>>,
}

impl From<TokenJSON> for Token {
  fn from(r: TokenJSON) -> Self {
  let kind = if r.value.is_some() { TokenKind::Token } else { TokenKind::Group };
  let extensions = match r.extensions {
    Some(map) if map.is_empty() => None,
    other => other,
  };
    Token {
      kind,
      token_type: r.token_type,
      description: r.description,
      value: r.value,
      deprecated: r.deprecated,
      extensions
    }
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(from = "TokenJSON")]
pub struct Token {
  pub kind: TokenKind,

  pub token_type: Option<String>,
  pub description: Option<String>,
  pub value: Option<Value>,
  pub deprecated: Option<bool>,
  pub extensions: Option<IndexMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
  Token,
  Group,
}

impl Default for TokenKind {
  fn default() -> Self {
    TokenKind::Token
  }
}

impl Token {
  pub fn new() -> Self {
    Self {
      kind: TokenKind::Token,
      token_type: None,
      description: None,
      value: None,
      deprecated: None,
      extensions: None,
    }
  }

  pub fn is_token(&self) -> bool {
    self.kind == TokenKind::Token
  }

  pub fn is_group(&self) -> bool {
    self.kind == TokenKind::Group
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_token_creation() {
    let token = Token::new();
    assert_eq!(token.kind, TokenKind::Token);
    assert!(token.token_type.is_none());
    assert!(token.description.is_none());
    assert!(token.value.is_none());
    assert!(token.deprecated.is_none());
    assert!(token.extensions.is_none());
  }

  #[test]
  fn test_token_kind_checks() {
    let token = Token::new();
    assert!(token.is_token());
    assert!(!token.is_group());

    let group = Token {
      kind: TokenKind::Group,
      ..Token::new()
    };
    assert!(!group.is_token());
    assert!(group.is_group());
  }

  #[test]
  fn test_token_from_json() {
    let json = r##"{
      "$type": "color",
      "$description": "Primary color",
      "$value": "#ff0000",
      "$deprecated": false
    }"##;

    let token: Token = serde_json::from_str(json).unwrap();

    assert_eq!(token.kind, TokenKind::Token);
    assert_eq!(token.token_type.unwrap(), "color");
    assert_eq!(token.description.unwrap(), "Primary color");
    assert_eq!(token.value.unwrap(), Value::String("#ff0000".to_string()));
    assert_eq!(token.deprecated.unwrap(), false);
    assert!(token.extensions.is_none());
  }
}