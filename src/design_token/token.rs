use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenValue {
  String(String),
  Number(f64),
  Bool(bool),
  Object(IndexMap<String, TokenValue>),
  Alias(String),
  Reference(String),
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