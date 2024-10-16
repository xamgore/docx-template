use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use docx_rust::document::{BodyContent, ParagraphContent, RunContent};
use hard_xml::{XmlWrite, XmlWriter};

use crate::fmt_to_io_adapter::IntoIoAdapter;

#[allow(clippy::large_enum_variant)]
#[allow(clippy::enum_variant_names)]
pub enum MarkupNode<'a> {
  InBody(BodyContent<'a>),
  InParagraph(ParagraphContent<'a>),
  InRun(RunContent<'a>),
}

impl<'a> Display for MarkupNode<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let mut w = XmlWriter::new(IntoIoAdapter::from(f));

    let result = match self {
      MarkupNode::InBody(n) => n.to_writer(&mut w),
      MarkupNode::InParagraph(n) => n.to_writer(&mut w),
      MarkupNode::InRun(n) => n.to_writer(&mut w),
    };

    result.map_err(|_| fmt::Error)
  }
}
