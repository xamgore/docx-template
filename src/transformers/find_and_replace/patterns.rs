use std::sync::Arc;

use aho_corasick::{BuildError, dfa, nfa};
use aho_corasick::automaton::Automaton;
use serde::Serialize;

#[derive(Clone)]
pub struct Patterns {
  pub automaton: Arc<dyn Automaton>,
}

impl Default for Patterns {
  fn default() -> Self {
    Self::from_iter::<&str, _>([])
  }
}

impl Patterns {
  /// Contains the automatic selection logic of the Aho-Corasick implementation to use.
  fn build<I, P>(patterns: I) -> Result<Arc<dyn Automaton>, BuildError>
    where
      I: IntoIterator<Item=P>,
      P: AsRef<[u8]>,
  {
    let nfa = nfa::noncontiguous::Builder::default().build(patterns)?;

    // zero-width patterns are not supported
    if nfa.min_pattern_len() == 0 {
      panic!("expected to have no empty patterns");
    }

    // We try to build a DFA if we have a very small number of patterns,
    // otherwise the memory usage just gets too crazy. We also only do it
    // when the start kind is unanchored or anchored, but not both, because
    // both implies two full copies of the transition table.
    if nfa.patterns_len() <= 100 {
      if let Ok(dfa) = dfa::Builder::default().build_from_noncontiguous(&nfa) {
        return Ok(Arc::new(dfa));
      }
    }

    // We basically always want a contiguous NFA if the limited
    // circumstances in which we use a DFA are not true. It is quite fast
    // and has excellent memory usage. The only way we don't use it is if
    // there are so many states that it can't fit in a contiguous NFA.
    // And the only way to know that is to try to build it. Building a
    // contiguous NFA is mostly just reshuffling data from a noncontiguous
    // NFA, so it isn't too expensive, especially relative to building a
    // noncontiguous NFA in the first place.
    if let Ok(cnfa) = nfa::contiguous::Builder::default().build_from_noncontiguous(&nfa) {
      Ok(Arc::new(cnfa))
    } else {
      Ok(Arc::new(nfa))
    }
  }
}

impl Patterns {
  /// Build patterns from an iterator.
  ///
  /// ```rust
  /// # use crate::docx_template::transformers::find_and_replace::Patterns;
  /// Patterns::from_iter(["{{id}}", "{{price}}", "{{consumer_name}}", "{{seller_name}}"]);
  /// ```
  #[allow(clippy::should_implement_trait)]
  pub fn from_iter<P: AsRef<[u8]>, I: IntoIterator<Item = P>>(
    patterns: I,
  ) -> Self {
    // as it fails only on extreme values, we unwrap for better api
    let automaton = Self::build(patterns).unwrap();
    Self { automaton }
  }

  /// Build patterns from an iterator.
  ///
  /// ```rust
  /// # use crate::docx_template::transformers::find_and_replace::Patterns;
  /// Patterns::from_iter_with_brackets("{{", "}}",
  ///   ["id", "price", "consumer_name", "seller_name"]);
  ///
  /// // same as
  /// Patterns::from_iter(["{{id}}", "{{price}}", "{{consumer_name}}", "{{seller_name}}"]);
  /// ```
  pub fn from_iter_with_brackets<P: AsRef<[u8]>, I: IntoIterator<Item = P>>(
    open_bracket: &str,
    close_bracket: &str,
    patterns: I,
  ) -> Self {
    let patterns = patterns.into_iter().map(|pattern| {
      let mut new = Vec::with_capacity(
        open_bracket.as_bytes().len() + close_bracket.as_bytes().len() + pattern.as_ref().len(),
      );
      new.extend_from_slice(open_bracket.as_bytes());
      new.extend_from_slice(pattern.as_ref());
      new.extend_from_slice(close_bracket.as_bytes());
      new
    });
    Self::from_iter(patterns)
  }

  /// Derive patterns from keys of a json object.
  ///
  /// ```rust
  /// use docx_template::transformers::find_and_replace::Patterns;
  /// use serde::Serialize;
  /// use serde_json::json;
  ///
  /// let value = json!({
  ///     "{{id}}": 42,
  ///     "{{price}}": 13.37,
  ///     "{{consumer_name}}": "Ryan Gosling",
  ///     "{{seller_name}}": "John Doe",
  /// });
  ///
  /// Patterns::from_json(&value);
  ///
  /// // same as
  /// Patterns::from_iter(["{{id}}", "{{price}}", "{{consumer_name}}", "{{seller_name}}"]);
  /// ```
  pub fn from_json(value: &serde_json::Value) -> Self {
    match value {
      serde_json::Value::Object(map) => Patterns::from_iter(map.keys()),
      _ => Patterns::default(),
    }
  }

  /// Derive patterns from keys of a json object.
  ///
  /// ```rust
  /// use docx_template::transformers::find_and_replace::Patterns;
  /// use serde::Serialize;
  /// use serde_json::json;
  ///
  /// let value = json!({
  ///     "id": 42,
  ///     "price": 13.37,
  ///     "consumer_name": "Ryan Gosling",
  ///     "seller_name": "John Doe",
  /// });
  ///
  /// Patterns::from_json_with_brackets("{{", "}}", &value);
  ///
  /// // same as
  /// Patterns::from_iter(["{{id}}", "{{price}}", "{{consumer_name}}", "{{seller_name}}"]);
  /// ```
  pub fn from_json_with_brackets(
    open_bracket: &str,
    close_bracket: &str,
    value: &serde_json::Value,
  ) -> Self {
    match value {
      serde_json::Value::Object(map) => {
        Patterns::from_iter_with_brackets(open_bracket, close_bracket, map.keys())
      }
      _ => Patterns::default(),
    }
  }

  /// Derive patterns from keys of a serializable `struct`.
  ///
  /// ```rust
  /// use docx_template::transformers::find_and_replace::Patterns;
  /// use serde::Serialize;
  ///
  /// #[derive(Default, Serialize)]
  /// struct Invoice {
  ///     #[serde(rename = "{{id}}")]
  ///     id: i64,
  ///     #[serde(rename = "{{price}}")]
  ///     price: f64,
  ///     #[serde(rename = "{{consumer}}")]
  ///     consumer: String,
  ///     #[serde(rename = "{{seller}}")]
  ///     seller: String,
  /// }
  ///
  /// Patterns::from_definition::<Invoice>().unwrap();
  ///
  /// // same as
  /// Patterns::from_iter(["{{id}}", "{{price}}", "{{consumer_name}}", "{{seller_name}}"]);
  /// ```
  pub fn from_definition<D: Default + Serialize>() -> Result<Self, serde_json::Error> {
    Ok(Self::from_json(&serde_json::to_value(D::default())?))
  }

  /// Derive patterns from keys of a serializable `struct`.
  ///
  /// ```rust
  /// use docx_template::transformers::find_and_replace::Patterns;
  /// use serde::Serialize;
  /// use serde_with::with_prefix;
  ///
  /// #[derive(Default, Serialize)]
  /// struct Invoice {
  ///     id: i64,
  ///     price: f64,
  ///     #[serde(flatten, with = "consumer_")]
  ///     consumer: Person,
  ///     #[serde(flatten, with = "seller_")]
  ///     seller: Person,
  /// }
  ///
  /// #[derive(Default, Serialize)]
  /// struct Person {
  ///     name: String,
  /// }
  ///
  /// serde_with::with_prefix!(consumer_ "consumer_");
  /// serde_with::with_prefix!(seller_ "seller_");
  ///
  /// Patterns::from_definition_with_brackets::<Invoice>("{{", "}}").unwrap();
  ///
  /// // same as
  /// Patterns::from_iter(["{{id}}", "{{price}}", "{{consumer_name}}", "{{seller_name}}"]);
  /// ```
  pub fn from_definition_with_brackets<D: Default + Serialize>(
    open_bracket: &str,
    close_bracket: &str,
  ) -> Result<Self, serde_json::Error> {
    let json = serde_json::to_value(D::default())?;
    Ok(Self::from_json_with_brackets(open_bracket, close_bracket, &json))
  }
}

impl<A: AsRef<[u8]>> FromIterator<A> for Patterns {
  fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
    Patterns::from_iter(iter)
  }
}

impl<'a> From<&'a serde_json::Value> for Patterns {
  fn from(value: &'a serde_json::Value) -> Self {
    Self::from_json(value)
  }
}

impl<'a> From<&'a serde_json::Map<String, serde_json::Value>> for Patterns {
  fn from(map: &'a serde_json::Map<String, serde_json::Value>) -> Self {
    Self::from_iter(map.keys())
  }
}
