#[cfg(feature = "docx-rs")]
use crate::DocxRsMarkupNode;
#[cfg(feature = "docx-rust")]
use crate::DocxRustMarkupNode;

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
  /// Replaces a placeholder with the markup node. Tables and images can be inserted this way.
  ///
  /// Be careful with reference ids, as they must not intersect with those already in the doc.
  ///
  /// Some markup nodes require storing _index_ information in a separate file of a `.docx` archive.
  /// Like images or comments. So it's easier to replace an already existing image, than to insert one.
  ///
  /// If you see how this can be simplified, feel free to [open an issue](https://github.com/xamgore/docx-template/issues/new).
  ///
  /// ```rust
  /// use docx_rust::document::{Break, BreakType, RunContent};
  /// use docx_template::{DocxRustMarkupNode, Replacements, Value};
  ///
  /// Replacements::from_slice([
  ///   Value::from_docx_rust_markup_node(DocxRustMarkupNode::InRun(
  ///     RunContent::Break(BreakType::Page.into())
  ///   ))
  /// ]);
  /// ```
  pub fn from_docx_rust_markup_node(node: DocxRustMarkupNode<'_>) -> Self {
    let xml = match node {
      DocxRustMarkupNode::InBody(_) => {
        // todo: this may not work as expected, as {placeholder} can reside at a table, not in the body
        format!("</w:t></w:r></w:p>{node}<w:p><w:r><w:t>")
      }
      DocxRustMarkupNode::InParagraph(_) => {
        format!("</w:t></w:r>{node}<w:r><w:t>")
      }
      DocxRustMarkupNode::InRun(_) => {
        format!("</w:t>{node}<w:t>")
      }
    };
    Self(xml)
  }

  #[cfg(feature = "docx-rs")]
  /// Replaces a placeholder with the markup node. Tables and images can be inserted this way.
  ///
  /// Be careful with reference ids, as they must not intersect with those already in the doc.
  /// `docx_rs` has several internal counters that are incremented each time a paragraph
  /// or another markup node is created. So it's better to avoid those counters,
  /// setting all the ids fields yourself.
  ///
  /// Some markup nodes require storing _index_ information in a separate file of a `.docx` archive.
  /// Like images or comments. So it's easier to replace an already existing image, than to insert one.
  ///
  /// If you see how this can be simplified, feel free to [open an issue](https://github.com/xamgore/docx-template/issues/new).
  ///
  /// ```rust
  /// use docx_rs::{Break, BreakType, RunChild};
  /// use docx_template::{DocxRsMarkupNode, Replacements, Value};
  ///
  /// Replacements::from_slice([
  ///   Value::from_docx_rs_markup_node(DocxRsMarkupNode::InRun(
  ///     RunChild::Break(Break::new(BreakType::Page))
  ///   ))
  /// ]);
  /// ```
  pub fn from_docx_rs_markup_node(node: DocxRsMarkupNode) -> Self {
    let xml = match node {
      DocxRsMarkupNode::InBody(_) => {
        // todo: this may not work as expected, as {placeholder} can reside at a table, not in the body
        format!("</w:t></w:r></w:p>{node}<w:p><w:r><w:t>")
      }
      DocxRsMarkupNode::InParagraph(_) => {
        format!("</w:t></w:r>{node}<w:r><w:t>")
      }
      DocxRsMarkupNode::InRun(_) => {
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
impl From<DocxRustMarkupNode<'_>> for Value {
  fn from(value: DocxRustMarkupNode<'_>) -> Self {
    Self::from_docx_rust_markup_node(value)
  }
}

#[cfg(feature = "docx-rs")]
impl From<DocxRsMarkupNode> for Value {
  fn from(value: DocxRsMarkupNode) -> Self {
    Self::from_docx_rs_markup_node(value)
  }
}

#[cfg(feature = "serde")]
impl From<serde_json::Value> for Value {
  fn from(value: serde_json::Value) -> Self {
    Self::from(&value)
  }
}

#[cfg(feature = "serde")]
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
