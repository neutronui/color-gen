use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[cfg(feature = "js")]
use boa_engine::{Context as JsContext, Source};

/// Represents the value of a design token, which can be a string, number,
/// bool, object, alias, reference, color, dimension, transform, or null.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenValue {
  String(String),
  Number(f64),
  Bool(bool),
  Object(IndexMap<String, TokenValue>),
  /// Alias to another token path (resolved eagerly to the target's value)
  Alias(String),
  /// Reference to another token path for CSS var generation (kept as var(--...))
  Reference(String),
  /// Color value (e.g., "#ff00aa", "rgb(0,0,0)", or a named color string)
  Color(String),
  /// A numeric value with a CSS unit (e.g., 4px, 1.5rem, 100%)
  Dimension { value: f64, unit: String },
  Transform(TransformExpr),
  Null,
}

/// Represents a design token with a name, value, and optional comment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
  pub name: String,
  pub value: TokenValue,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub comment: Option<String>,
}

pub type TokenSet = IndexMap<String, Token>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformStep {
  pub r#type: String,
  pub args: Vec<TokenValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformExpr {
  pub steps: Vec<TransformStep>,
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

// A pluggable registry for transformation functions.
type TransformFn = fn(&TransformRegistry, &mut TransformContext, Option<TokenValue>, &TransformStep) -> Result<TokenValue, ResolveError>;

pub struct TransformRegistry {
  builtins: IndexMap<String, TransformFn>,
  #[cfg(feature = "js")]
  js_funcs: IndexMap<String, String>, // name -> JS function source
}

impl Default for TransformRegistry {
  fn default() -> Self {
    let mut builtins: IndexMap<String, TransformFn> = IndexMap::new();
    // Register built-in transforms
    builtins.insert("alias".into(), builtin_alias as TransformFn);
    builtins.insert("multiply".into(), builtin_multiply as TransformFn);
    builtins.insert("add".into(), builtin_add as TransformFn);
    builtins.insert("subtract".into(), builtin_subtract as TransformFn);
    builtins.insert("divide".into(), builtin_divide as TransformFn);

    TransformRegistry {
      builtins,
      #[cfg(feature = "js")]
      js_funcs: IndexMap::new(),
    }
  }
}

impl TransformRegistry {
  pub fn get_builtin(&self, name: &str) -> Option<&TransformFn> {
    self.builtins.get(name)
  }

  #[cfg(feature = "js")]
  pub fn add_js_transform(&mut self, name: &str, source: &str) {
    self.js_funcs.insert(name.to_string(), source.to_string());
  }
}

pub struct TransformContext<'a> {
  pub name: &'a str,
  pub tokens: &'a TokenSet,
  pub resolved: &'a mut TokenSet,
  pub stack: &'a mut Vec<String>,
}

pub fn resolve_tokens(tokens: &TokenSet) -> Result<TokenSet, ResolveError> {
  let registry = TransformRegistry::default();
  resolve_tokens_with_registry(tokens, &registry)
}

pub fn resolve_tokens_with_registry(tokens: &TokenSet, registry: &TransformRegistry) -> Result<TokenSet, ResolveError> {
  let mut resolved: TokenSet = IndexMap::new();
  let mut stack: Vec<String> = Vec::new();

  for (key, token) in tokens.iter() {
    if resolved.contains_key(key) {
      continue;
    }

    stack.clear();
    stack.push(key.clone());

    let val = resolve_value(key, &token.value, tokens, &mut resolved, &mut stack, registry)
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
  stack: &mut Vec<String>,
  registry: &TransformRegistry,
) -> Result<TokenValue, ResolveError> {
  let mut current: Option<TokenValue> = None;
  for step in &expr.steps {
    current = Some(apply_transform_step(
      name,
      step,
      current.clone(),
      tokens,
      resolved,
      stack,
      registry,
    )?);
  }
  Ok(current.unwrap_or(TokenValue::Null))
}

fn resolve_value(
  name: &str,
  val: &TokenValue,
  tokens: &TokenSet,
  resolved: &mut TokenSet,
  stack: &mut Vec<String>,
  registry: &TransformRegistry,
) -> Result<TokenValue, ResolveError> {
  match val {
    TokenValue::Transform(expr) => apply_transform_pipeline(name, expr, tokens, resolved, stack, registry),
    TokenValue::Reference(target_path) => {
      if !tokens.contains_key(target_path) {
        return Err(ResolveError::TokenNotFound(format!(
          "{} (referenced by {})",
          target_path, name
        )));
      }
      let css_var = format!("--{}", target_path.replace('.', "-"));
      Ok(TokenValue::String(format!("var({})", css_var)))
    }
    TokenValue::Alias(target_path) => resolve_alias(name, target_path, tokens, resolved, stack, registry),
    TokenValue::Object(map) => {
      let mut new_map = IndexMap::new();
      for (k, v) in map.iter() {
        let rv = resolve_value(name, v, tokens, resolved, stack, registry)?;
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
  key: &str,
) -> Option<String> {
  if let Some(t) = resolved.get(key) {
    t.comment.clone()
  } else {
    tokens.get(key).and_then(|t| t.comment.clone())
  }
}

fn resolve_alias(
  name: &str,
  target_path: &str,
  tokens: &TokenSet,
  resolved: &mut TokenSet,
  stack: &mut Vec<String>,
  registry: &TransformRegistry,
) -> Result<TokenValue, ResolveError> {
  if stack.contains(&target_path.to_string()) {
    return Err(ResolveError::CycleDetected(target_path.to_string()));
  }

  let target_token = tokens.get(target_path).ok_or_else(|| {
    ResolveError::TokenNotFound(format!("{} (referenced by {})", target_path, name))
  })?;

  if let Some(resolved_token) = resolved.get(target_path) {
    return Ok(resolved_token.value.clone());
  }

  stack.push(target_path.to_string());
  let resolved_value = resolve_value(
    target_path,
    &target_token.value,
    tokens,
    resolved,
    stack,
    registry,
  )?;
  stack.pop();

  let new_token = Token {
    name: target_path.to_string(),
    value: resolved_value.clone(),
    comment: resolved_token_comment(resolved, tokens, target_path),
  };
  resolved.insert(target_path.to_string(), new_token);
  Ok(resolved_value)
}

// --- helpers for CSS calc math ---
fn is_css_calcable_string(s: &str) -> bool {
  let trimmed = s.trim_start();
  trimmed.starts_with("var(") || trimmed.starts_with("calc(")
}

fn fmt_num(n: f64) -> String {
  if (n.fract()).abs() < std::f64::EPSILON {
    format!("{:.0}", n)
  } else {
    n.to_string()
  }
}

fn css_calc(expr_lhs: &str, op: &str, expr_rhs: &str) -> String {
  format!("calc({} {} {})", expr_lhs, op, expr_rhs)
}

// ---- Built-in transform implementations ----
fn builtin_alias(
  _registry: &TransformRegistry,
  ctx: &mut TransformContext,
  _input: Option<TokenValue>,
  step: &TransformStep,
) -> Result<TokenValue, ResolveError> {
  let target = step
    .args
    .get(0)
    .and_then(|v| match v {
      TokenValue::String(s) => Some(s.clone()),
      TokenValue::Alias(s) => Some(s.clone()),
      _ => None,
    })
    .ok_or_else(|| ResolveError::InvalidTransform("alias requires string arg".into()))?;
  resolve_alias(ctx.name, &target, ctx.tokens, ctx.resolved, ctx.stack, _registry)
}

fn builtin_multiply(
  _registry: &TransformRegistry,
  _ctx: &mut TransformContext,
  input: Option<TokenValue>,
  step: &TransformStep,
) -> Result<TokenValue, ResolveError> {
  let factor = match step.args.get(0) {
    Some(TokenValue::Number(n)) => *n,
    _ => return Err(ResolveError::InvalidTransform("multiply expects number".into())),
  };

  match input {
    Some(TokenValue::Number(n)) => Ok(TokenValue::Number(n * factor)),
    Some(TokenValue::Dimension { value, unit }) => Ok(TokenValue::Dimension { value: value * factor, unit }),
    Some(TokenValue::String(s)) if is_css_calcable_string(&s) => Ok(TokenValue::String(css_calc(&s, "*", &fmt_num(factor)))),
    Some(TokenValue::String(s)) => Ok(TokenValue::String(css_calc(&s, "*", &fmt_num(factor)))),
    _ => Err(ResolveError::TransformFailed("multiply expects number/dimension/string input".into())),
  }
}

fn builtin_add(
  _registry: &TransformRegistry,
  _ctx: &mut TransformContext,
  input: Option<TokenValue>,
  step: &TransformStep,
) -> Result<TokenValue, ResolveError> {
  let (add_val, add_unit_opt): (f64, Option<String>) = match step.args.get(0) {
    Some(TokenValue::Number(n)) => (*n, None),
    Some(TokenValue::Dimension { value, unit }) => (*value, Some(unit.clone())),
    _ => return Err(ResolveError::InvalidTransform("add expects number or dimension".into())),
  };

  match input {
    Some(TokenValue::Number(n)) => Ok(TokenValue::Number(n + add_val)),
    Some(TokenValue::Dimension { value, unit }) => {
      if let Some(u) = &add_unit_opt {
        if u != &unit {
          return Err(ResolveError::TransformFailed("add unit mismatch".into()));
        }
      }
      Ok(TokenValue::Dimension { value: value + add_val, unit })
    }
    Some(TokenValue::String(s)) if is_css_calcable_string(&s) => {
      let rhs = if let Some(u) = add_unit_opt { format!("{}{}", fmt_num(add_val), u) } else { fmt_num(add_val) };
      Ok(TokenValue::String(css_calc(&s, "+", &rhs)))
    }
    Some(TokenValue::String(s)) => {
      let rhs = if let Some(u) = add_unit_opt { format!("{}{}", fmt_num(add_val), u) } else { fmt_num(add_val) };
      Ok(TokenValue::String(css_calc(&s, "+", &rhs)))
    }
    _ => Err(ResolveError::TransformFailed("add expects number/dimension/string input".into())),
  }
}

fn builtin_subtract(
  _registry: &TransformRegistry,
  _ctx: &mut TransformContext,
  input: Option<TokenValue>,
  step: &TransformStep,
) -> Result<TokenValue, ResolveError> {
  let (sub_val, sub_unit_opt): (f64, Option<String>) = match step.args.get(0) {
    Some(TokenValue::Number(n)) => (*n, None),
    Some(TokenValue::Dimension { value, unit }) => (*value, Some(unit.clone())),
    _ => return Err(ResolveError::InvalidTransform("subtract expects number or dimension".into())),
  };

  match input {
    Some(TokenValue::Number(n)) => Ok(TokenValue::Number(n - sub_val)),
    Some(TokenValue::Dimension { value, unit }) => {
      if let Some(u) = &sub_unit_opt {
        if u != &unit {
          return Err(ResolveError::TransformFailed("subtract unit mismatch".into()));
        }
      }
      Ok(TokenValue::Dimension { value: value - sub_val, unit })
    }
    Some(TokenValue::String(s)) if is_css_calcable_string(&s) => {
      let rhs = if let Some(u) = sub_unit_opt { format!("{}{}", fmt_num(sub_val), u) } else { fmt_num(sub_val) };
      Ok(TokenValue::String(css_calc(&s, "-", &rhs)))
    }
    Some(TokenValue::String(s)) => {
      let rhs = if let Some(u) = sub_unit_opt { format!("{}{}", fmt_num(sub_val), u) } else { fmt_num(sub_val) };
      Ok(TokenValue::String(css_calc(&s, "-", &rhs)))
    }
    _ => Err(ResolveError::TransformFailed("subtract expects number/dimension/string input".into())),
  }
}

fn builtin_divide(
  _registry: &TransformRegistry,
  _ctx: &mut TransformContext,
  input: Option<TokenValue>,
  step: &TransformStep,
) -> Result<TokenValue, ResolveError> {
  let divisor = match step.args.get(0) {
    Some(TokenValue::Number(n)) => *n,
    _ => return Err(ResolveError::InvalidTransform("divide expects number".into())),
  };
  if divisor == 0.0 {
    return Err(ResolveError::TransformFailed("divide by zero".into()));
  }

  match input {
    Some(TokenValue::Number(n)) => Ok(TokenValue::Number(n / divisor)),
    Some(TokenValue::Dimension { value, unit }) => Ok(TokenValue::Dimension { value: value / divisor, unit }),
    Some(TokenValue::String(s)) if is_css_calcable_string(&s) => Ok(TokenValue::String(css_calc(&s, "/", &fmt_num(divisor)))),
    Some(TokenValue::String(s)) => Ok(TokenValue::String(css_calc(&s, "/", &fmt_num(divisor)))),
    _ => Err(ResolveError::TransformFailed("divide expects number/dimension/string input".into())),
  }
}

#[cfg(feature = "js")]
fn tokenvalue_to_json(val: &TokenValue) -> serde_json::Value {
  match val {
    TokenValue::Null => serde_json::Value::Null,
    TokenValue::Bool(b) => serde_json::Value::Bool(*b),
    TokenValue::Number(n) => serde_json::json!(n),
    TokenValue::String(s) | TokenValue::Color(s) => serde_json::Value::String(s.clone()),
    TokenValue::Dimension { value, unit } => serde_json::json!({"type":"dimension","value":value,"unit":unit}),
    TokenValue::Object(map) => {
      let mut m = serde_json::Map::new();
      for (k, v) in map.iter() { m.insert(k.clone(), tokenvalue_to_json(v)); }
      serde_json::Value::Object(m)
    }
    TokenValue::Alias(s) => serde_json::json!({"alias": s}),
    TokenValue::Reference(s) => serde_json::json!({"reference": s}),
    TokenValue::Transform(expr) => serde_json::to_value(expr).unwrap_or(serde_json::Value::Null),
  }
}

#[cfg(feature = "js")]
fn json_to_tokenvalue_value(val: serde_json::Value) -> TokenValue {
  match val {
    serde_json::Value::Null => TokenValue::Null,
    serde_json::Value::Bool(b) => TokenValue::Bool(b),
    serde_json::Value::Number(n) => TokenValue::Number(n.as_f64().unwrap_or(0.0)),
    serde_json::Value::String(s) => TokenValue::String(s),
    serde_json::Value::Array(arr) => {
      let mut m = IndexMap::new();
      for (i, v) in arr.into_iter().enumerate() { m.insert(i.to_string(), json_to_tokenvalue_value(v)); }
      TokenValue::Object(m)
    }
    serde_json::Value::Object(mut obj) => {
      if let (Some(t), Some(v), Some(u)) = (
        obj.get("type").and_then(|v| v.as_str()).map(|s| s.to_string()),
        obj.get("value").cloned(),
        obj.get("unit").and_then(|v| v.as_str()).map(|s| s.to_string()),
      ) {
        if t == "dimension" {
          return TokenValue::Dimension { value: v.as_f64().unwrap_or(0.0), unit: u };
        }
      }
      let mut m = IndexMap::new();
      for (k, v) in obj.into_iter() { m.insert(k, json_to_tokenvalue_value(v)); }
      TokenValue::Object(m)
    }
  }
}

#[cfg(feature = "js")]
fn run_js_transform(
  name: &str,
  source: &str,
  token_name: &str,
  input: Option<TokenValue>,
  step: &TransformStep,
  _tokens: &TokenSet,
  _resolved: &mut TokenSet,
  _stack: &mut Vec<String>,
) -> Result<TokenValue, ResolveError> {
  let input_json = tokenvalue_to_json(&input.unwrap_or(TokenValue::Null)).to_string();
  let args_json = serde_json::to_string(&step.args).unwrap_or_else(|_| "[]".into());
  let ctx_json = serde_json::json!({"token": token_name}).to_string();
  let call_script = format!(
    "{}\n(function(){{ const fn = (typeof {}==='function'? {} : globalThis[{}]); if(!fn) throw new Error('transform not found'); return JSON.stringify(fn({}, {}, {})); }})()",
    source,
    name,
    name,
    serde_json::to_string(name).unwrap(),
    input_json,
    args_json,
    ctx_json
  );
  let mut ctx = JsContext::default();
  let result = ctx
    .eval(Source::from_bytes(&call_script))
    .map_err(|e| ResolveError::TransformFailed(format!("JS eval error in '{}': {}", name, e)))?;
  let s = result
    .as_string()
    .ok_or_else(|| ResolveError::TransformFailed("JS transform did not return a JSON string".into()))?
    .to_std_string()
    .map_err(|_| ResolveError::TransformFailed("failed to convert JS string".into()))?;
  let val: serde_json::Value = serde_json::from_str(&s)
    .map_err(|e| ResolveError::TransformFailed(format!("JS returned invalid JSON: {}", e)))?;
  Ok(json_to_tokenvalue_value(val))
}
fn apply_transform_step(
  name: &str,
  step: &TransformStep,
  input: Option<TokenValue>,
  tokens: &TokenSet,
  resolved: &mut TokenSet,
  stack: &mut Vec<String>,
  registry: &TransformRegistry,
) -> Result<TokenValue, ResolveError> {
  // Dispatch to built-in or JS transform by name
  if let Some(func) = registry.get_builtin(step.r#type.as_str()) {
    let mut ctx = TransformContext { name, tokens, resolved, stack };
    return func(registry, &mut ctx, input, step);
  }

  #[cfg(feature = "js")]
  {
    // Evaluate all registered JS sources and then call the function named by step.r#type.
    // This allows plugins to register multiple transforms in a single file.
    if !registry.js_funcs.is_empty() {
      // Concatenate sources and execute once
      let combined = registry
        .js_funcs
        .values()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n\n");
      // Try to invoke the target function
      if let Ok(rv) = run_js_transform(step.r#type.as_str(), &combined, name, input.clone(), step, tokens, resolved, stack) {
        return Ok(rv);
      }
    }
  }

  Err(ResolveError::InvalidTransform(format!("unknown transform: {}", step.r#type)))
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

pub fn token_value_to_string(value: &TokenValue) -> String {
  match value {
    TokenValue::String(s) => s.clone(),
    TokenValue::Number(n) => {
      if (n.fract()).abs() < std::f64::EPSILON {
        format!("{:.0}", n)
      } else {
        n.to_string()
      }
    }
    TokenValue::Bool(b) => b.to_string(),
    TokenValue::Object(obj) => serde_json::to_string(obj).unwrap_or_else(|_| String::from("{}")),
    TokenValue::Alias(a) => format!("alias({})", a),
    TokenValue::Reference(r) => format!("reference({})", r),
    TokenValue::Color(c) => c.clone(),
    TokenValue::Dimension { value, unit } => {
      if (value.fract()).abs() < std::f64::EPSILON {
        format!("{:.0}{}", value, unit)
      } else {
        format!("{}{}", value, unit)
      }
    }
    TokenValue::Transform(_) => "unresolved-transform".to_string(),
    TokenValue::Null => String::from("null"),
  }
}

/// Build a CSS stylesheet with a selector (e.g., ":root") and optional prefix for variable names.
pub fn to_css_stylesheet(tokens: &TokenSet, selector: &str, prefix: Option<&str>) -> String {
  let mut out = String::new();
  out.push_str(selector);
  out.push_str(" {\n");
  for (key, token) in tokens.iter() {
    let var_name = match prefix {
      Some(p) if !p.is_empty() => format!("--{}-{}", p, key.replace('.', "-")),
      _ => format!("--{}", key.replace('.', "-")),
    };
    let value = token_value_to_string(&token.value);
    out.push_str("  ");
    out.push_str(&var_name);
    out.push_str(": ");
    out.push_str(&value);
    out.push_str(";\n");
  }
  out.push('}');
  out
}

/// Produce a mapping from token path (e.g., "spacing.base") to resolved string value.
pub fn to_resolved_string_map(tokens: &TokenSet) -> IndexMap<String, String> {
  let mut map = IndexMap::new();
  for (key, token) in tokens.iter() {
    map.insert(key.clone(), token_value_to_string(&token.value));
  }
  map
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
      comment: Some("A bright blue color".to_string()),
    },
  );

  tokens.insert(
    "color.brand.50".to_string(),
    Token {
      name: "color.brand.50".to_string(),
      value: TokenValue::Reference("color.blue.50".to_string()),
      comment: Some("Brand color aliasing blue".to_string()),
    },
  );

  tokens.insert(
    "spacing.base".to_string(),
    Token {
      name: "spacing.base".to_string(),
      value: TokenValue::Dimension { value: 4.0, unit: "px".to_string() },
      comment: None,
    },
  );

  tokens.insert(
    "spacing.large".to_string(),
    Token {
      name: "spacing.large".to_string(),
      value: TokenValue::Transform(TransformExpr {
        steps: vec![
          TransformStep { r#type: "alias".to_string(), args: vec![TokenValue::String("spacing.base".to_string())] },
          TransformStep { r#type: "multiply".to_string(), args: vec![TokenValue::Number(4.0)] },
        ],
      }),
      comment: Some("Large spacing as 4x base".to_string()),
    },
  );

  // Demonstrate var() + calc(): create a token that references spacing.base,
  // then scale it via transform
  tokens.insert(
    "spacing.base.ref".to_string(),
    Token {
      name: "spacing.base.ref".to_string(),
      value: TokenValue::Reference("spacing.base".to_string()),
      comment: Some("CSS var reference to spacing.base".to_string()),
    },
  );

  tokens.insert(
    "spacing.huge".to_string(),
    Token {
      name: "spacing.huge".to_string(),
      value: TokenValue::Transform(TransformExpr {
        steps: vec![
          TransformStep { r#type: "alias".to_string(), args: vec![TokenValue::String("spacing.base.ref".to_string())] },
          TransformStep { r#type: "multiply".to_string(), args: vec![TokenValue::Number(3.0)] },
          TransformStep { r#type: "add".to_string(), args: vec![TokenValue::Dimension { value: 2.0, unit: "px".to_string() }] },
        ],
      }),
      comment: Some("Scaled ref with calc()".to_string()),
    },
  );

  let resolved = resolve_tokens(&tokens)?;
  let css_map = to_css_custom_properties(&resolved);

  for (name, value) in css_map.iter() {
    println!("{}: {};", name, value);
  }

  Ok(())
}