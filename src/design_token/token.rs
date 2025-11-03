use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::{Error as DeError, Unexpected};

#[derive(Debug, Clone, PartialEq, Serialize)]
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

impl<'de> Deserialize<'de> for TokenValue {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    // Strategy: Deserialize into an intermediate serde_json::Value, then interpret.
    let v: serde_json::Value = serde_json::Value::deserialize(deserializer)?;
    Ok(match v {
      serde_json::Value::Null => TokenValue::Null,
      serde_json::Value::Bool(b) => TokenValue::Bool(b),
      serde_json::Value::Number(n) => TokenValue::Number(n.as_f64().ok_or_else(|| D::Error::invalid_type(Unexpected::Other("non-f64 number"), &"f64"))?),
      serde_json::Value::String(s) => TokenValue::String(s),
      serde_json::Value::Array(arr) => {
        // Represent arrays as Object with numeric keys to preserve content
        let mut map = IndexMap::new();
        for (i, item) in arr.into_iter().enumerate() {
          let tv = serde_json::from_value::<TokenValue>(item).map_err(|e| D::Error::custom(e.to_string()))?;
          map.insert(i.to_string(), tv);
        }
        TokenValue::Object(map)
      }
      serde_json::Value::Object(obj) => {
        // Detect alias/reference tagged objects
        if let Some(alias_val) = obj.get("alias") {
          if let Some(s) = alias_val.as_str() {
            TokenValue::Alias(s.to_string())
          } else {
            return Err(D::Error::invalid_type(Unexpected::Other("non-string alias"), &"alias must be a string"));
          }
        } else if let Some(reference_val) = obj.get("reference") {
          if let Some(s) = reference_val.as_str() {
            TokenValue::Reference(s.to_string())
          } else {
            return Err(D::Error::invalid_type(Unexpected::Other("non-string reference"), &"reference must be a string"));
          }
        } else {
          // Recursively convert object values to TokenValue
          let mut map = IndexMap::new();
          for (k, val) in obj.into_iter() {
            let tv = serde_json::from_value::<TokenValue>(val).map_err(|e| D::Error::custom(e.to_string()))?;
            map.insert(k, tv);
          }
          TokenValue::Object(map)
        }
      }
    })
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
  pub name: String,
  pub value: TokenValue,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub comment: Option<String>
}

pub type TokenSet = IndexMap<String, Token>;