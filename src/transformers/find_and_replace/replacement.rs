use serde_json::Value;
use std::borrow::Cow;
use itertools::Itertools;

#[cfg(feature = "docx-rust")]
use crate::doc_layout_node::DocLayoutNode;

#[derive(Debug, Clone)]
pub enum Replacement<'s> {
  Xml(Cow<'s, str>),
  // todo: "dynamic values" — functions returning a value, needs real-life use cases though
  // todo: images, as they require records in index files
  //       https://web.archive.org/web/20220626225526/http://officeopenxml.com/drwPic-ImageData.php
  //       and have quite a cumbersome markup
  //       https://web.archive.org/web/20220627000043/http://officeopenxml.com/drwPic.php
}

impl<'s> Replacement<'s> {
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
        // this may not work as expected, as {placeholder} can reside at a table, not in the body
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

impl<'s> From<&'s str> for Replacement<'s> {
  fn from(value: &str) -> Self {
    Self::from_text(value)
  }
}

#[cfg(feature = "docx-rust")]
impl<'s> From<DocLayoutNode<'_>> for Replacement<'s> {
  fn from(value: DocLayoutNode<'_>) -> Self {
    Self::from_layout_node(value)
  }
}

impl<'s> From<Value> for Replacement<'s> {
  fn from(value: Value) -> Self {
    match value {
      Value::Null => Replacement::Xml(Cow::default()),
      Value::String(v) => Replacement::from_text(&v),
      Value::Number(v) => Replacement::from_text(&v.to_string()),
      _ => unimplemented!(),
    }
  }
}
