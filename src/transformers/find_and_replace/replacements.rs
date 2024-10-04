use super::value::Value;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::borrow::Cow;
use std::ops::Index;

/// Struct of values to fill in placeholders during rendering.
#[derive(Debug, Default, Clone)]
pub struct Replacements<'a> {
  /// The order is important as indexes are encoded at `Placeholders`' automatons
  values: Cow<'a, [Value<'a>]>,
}

impl<'a> Replacements<'a> {
  pub fn from_slice<I: Into<Cow<'a, [Value<'a>]>>>(slice: I) -> Self {
    Self { values: slice.into() }
  }

  #[allow(clippy::should_implement_trait)]
  pub fn from_iter<V: Into<Value<'a>>, I: IntoIterator<Item = V>>(iter: I) -> Self {
    Self { values: iter.into_iter().map(Into::into).collect() }
  }

  pub fn from_json(object: &'a JsonValue) -> Self {
    debug_assert!(object.is_object(), "pass an object, as placeholders won't be replaced otherwise");
    match object.as_object() {
      Some(obj) => Self { values: obj.values().map(Value::from).collect() },
      None => Default::default(),
    }
  }
}

impl Replacements<'static> {
  pub fn try_from_serializable(val: &impl Serialize) -> Result<Self, serde_json::Error> {
    let json = serde_json::to_value(val)?;
    debug_assert!(json.is_object(), "pass an object, as placeholders won't be replaced otherwise");
    Ok(match json.as_object() {
      Some(obj) => Self { values: obj.values().cloned().map(Value::from).collect() },
      None => Default::default(),
    })
  }
}

impl<'a> Index<usize> for Replacements<'a> {
  type Output = Value<'a>;

  fn index(&self, index: usize) -> &Self::Output {
    &self.values[index]
  }
}
