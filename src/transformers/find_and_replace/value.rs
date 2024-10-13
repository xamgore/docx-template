#[cfg(feature = "docx-rust")]
use crate::markup_node::MarkupNode;

/// A text value or a piece of XML ready to replace a placeholder.
#[derive(Debug, Default, Clone)]
pub struct Value(pub(crate) String);

impl Value {
  #[allow(missing_docs)]
  pub fn from_xml(xml: impl Into<String>) -> Self {
    Self(xml.into())
  }

  /// Replaces a placeholder with the text. Each `\n` or `\r\n` symbol forms a new line in the document.
  pub fn from_text(text: &str) -> Self {
    let lines = text.lines().map(quick_xml::escape::escape);
    Self(crate::iter_tools::join(lines, "</w:t><w:br/><w:t>"))
  }

  #[cfg(feature = "docx-rust")]
  /// Replaces a placeholder with the markup node. Tables or images can be inserted this way.
  ///
  /// ```rust
  /// use docx_rust::document::{Break, BreakType, RunContent};
  /// use docx_template::{MarkupNode, Replacements, Value};
  ///
  /// Replacements::from_slice([
  ///   Value::from_layout_node(MarkupNode::InRun(
  ///     RunContent::Break(BreakType::Page.into())
  ///   ))
  /// ]);
  /// ```
  pub fn from_layout_node(node: MarkupNode<'_>) -> Self {
    let xml = match node {
      MarkupNode::InBody(_) => {
        // todo: this may not work as expected, as {placeholder} can reside at a table, not in the body
        format!("</w:t></w:r></w:p>{node}<w:p><w:r><w:t>")
      }
      MarkupNode::InParagraph(_) => {
        format!("</w:t></w:r>{node}<w:r><w:t>")
      }
      MarkupNode::InRun(_) => {
        format!("</w:t>{node}<w:t>")
      }
    };
    Self(xml)
  }
}

impl From<&str> for Value {
  fn from(value: &str) -> Self {
    Self::from_text(value)
  }
}

#[cfg(feature = "docx-rust")]
impl From<MarkupNode<'_>> for Value {
  fn from(value: MarkupNode<'_>) -> Self {
    Self::from_layout_node(value)
  }
}

impl From<serde_json::Value> for Value {
  fn from(value: serde_json::Value) -> Self {
    Self::from(&value)
  }
}

impl From<&serde_json::Value> for Value {
  fn from(value: &serde_json::Value) -> Self {
    match value {
      serde_json::Value::Null => Value::from_xml(String::new()),
      serde_json::Value::String(v) => Value::from_text(v.as_str()),
      serde_json::Value::Number(v) => Value::from_text(&v.to_string()),
      _ if cfg!(debug_assertions) => unimplemented!(),
      _ => Value::from_xml(String::new()),
    }
  }
}
