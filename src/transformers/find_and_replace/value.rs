use std::borrow::Cow;
use itertools::Itertools;

#[cfg(feature = "docx-rust")]
use crate::doc_layout_node::DocLayoutNode;

#[derive(Debug, Clone)]
pub enum Value<'s> {
  Xml(Cow<'s, str>),
  // todo: "dynamic values" — functions returning a value, needs real-life use cases though
  // todo: images, as they require records in index files
  //       https://web.archive.org/web/20220626225526/http://officeopenxml.com/drwPic-ImageData.php
  //       and have quite a cumbersome markup
  //       https://web.archive.org/web/20220627000043/http://officeopenxml.com/drwPic.php
}

impl<'s> Value<'s> {
  pub fn from_xml<I: Into<Cow<'s, str>>>(xml: I) -> Self {
    Self::Xml(xml.into())
  }

  pub fn from_text(text: &str) -> Self {
    Self::Xml(Cow::Owned(
      regex!(r#"\r?\n"#) // avoid line break collapse
        .split(text)
        .map(quick_xml::escape::escape)
        .join("</w:t><w:br/><w:t>"),
    ))
  }

  #[cfg(feature = "docx-rust")]
  pub fn from_layout_node(node: DocLayoutNode<'_>) -> Self {
    let xml = match node {
      DocLayoutNode::InBody(_) => {
        // todo: this may not work as expected, as {placeholder} can reside at a table, not in the body
        format!("</w:t></w:r></w:p>{node}<w:p><w:r><w:t>")
      }
      DocLayoutNode::InParagraph(_) => {
        format!("</w:t></w:r>{node}<w:r><w:t>")
      }
      DocLayoutNode::InRun(_) => {
        format!("</w:t>{node}<w:t>")
      }
    };
    Self::Xml(Cow::Owned(xml))
  }
}

impl<'s> From<&'s str> for Value<'s> {
  fn from(value: &str) -> Self {
    Self::from_text(value)
  }
}

#[cfg(feature = "docx-rust")]
impl<'s> From<DocLayoutNode<'_>> for Value<'s> {
  fn from(value: DocLayoutNode<'_>) -> Self {
    Self::from_layout_node(value)
  }
}

impl<'s> From<serde_json::Value> for Value<'s> {
  fn from(value: serde_json::Value) -> Self {
    match value {
      serde_json::Value::Null => Value::Xml(Cow::default()),
      serde_json::Value::String(v) => Value::from_text(&v),
      serde_json::Value::Number(v) => Value::from_text(&v.to_string()),
      _ => unimplemented!(),
    }
  }
}

impl<'s> From<&'s serde_json::Value> for Value<'s> {
  fn from(value: &'s serde_json::Value) -> Self {
    match value {
      serde_json::Value::Null => Value::Xml(Cow::default()),
      serde_json::Value::String(v) => Value::from_text(v.as_str()),
      serde_json::Value::Number(v) => Value::from_text(&v.to_string()),
      _ => unimplemented!(),
    }
  }
}
