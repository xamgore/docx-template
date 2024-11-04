use crate::fmt_to_io_adapter::IntoIoAdapter;
use std::fmt::{self, Display, Formatter};

#[cfg(feature = "docx-rust")]
pub mod docx_rust {
  use super::*;
  use ::docx_rust::document::{BodyContent, ParagraphContent, RunContent};
  use hard_xml::{XmlWrite, XmlWriter};

  #[allow(clippy::large_enum_variant)]
  #[allow(clippy::enum_variant_names)]
  #[allow(missing_docs)]
  pub enum DocxRustMarkupNode<'a> {
    /// Body's child.
    InBody(Vec<BodyContent<'a>>),
    /// Paragraph's child.
    InParagraph(Vec<ParagraphContent<'a>>),
    /// Run's child.
    InRun(Vec<RunContent<'a>>),
  }

  impl<'a> Display for DocxRustMarkupNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
      let mut w = XmlWriter::new(IntoIoAdapter::from(f));

      let result = match self {
        DocxRustMarkupNode::InBody(children) => {
          children.iter().try_for_each(|ch| ch.to_writer(&mut w))
        }
        DocxRustMarkupNode::InParagraph(children) => {
          children.iter().try_for_each(|ch| ch.to_writer(&mut w))
        }
        DocxRustMarkupNode::InRun(children) => {
          children.iter().try_for_each(|ch| ch.to_writer(&mut w))
        }
      };

      result.map_err(|_| fmt::Error)
    }
  }
}

#[cfg(feature = "docx-rs")]
pub mod docx_rs {
  use super::*;
  use ::docx_rs::{BuildXML, DocumentChild, ParagraphChild, RunChild};
  use ::xml::EmitterConfig;

  #[allow(clippy::large_enum_variant)]
  #[allow(clippy::enum_variant_names)]
  #[allow(missing_docs)]
  pub enum DocxRsMarkupNode {
    /// Body's child.
    InBody(Vec<DocumentChild>),
    /// Paragraph's child.
    InParagraph(Vec<ParagraphChild>),
    /// Run's child.
    InRun(Vec<RunChild>),
  }

  impl Display for DocxRsMarkupNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
      let config = EmitterConfig {
        write_document_declaration: false,
        perform_escaping: false,
        perform_indent: false,
        line_separator: "".into(),
        ..Default::default()
      };

      let w = config.create_writer(IntoIoAdapter::from(f));

      let result = match self {
        DocxRsMarkupNode::InBody(children) => {
          children.iter().try_fold(w, |w, child| child.build_to(w))
        }
        DocxRsMarkupNode::InParagraph(children) => {
          children.iter().try_fold(w, |w, child| child.build_to(w))
        }
        DocxRsMarkupNode::InRun(children) => {
          children.iter().try_fold(w, |w, child| child.build_to(w))
        }
      };

      result.map_err(|_| fmt::Error)?;
      Ok(())
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;
    use ::docx_rs::{Break, BreakType};

    #[test]
    fn test_body_prefix_and_suffix_have_not_changed_when_bumping_docs_rs_version() {
      let actual =
        DocxRsMarkupNode::InBody(vec![DocumentChild::Paragraph(Default::default())]).to_string();
      assert_eq!(actual, r#"<w:p w14:paraId="00000001"><w:pPr><w:rPr /></w:pPr></w:p>"#)
    }

    #[test]
    fn test_paragraph_prefix_and_suffix_have_not_changed_when_bumping_docs_rs_version() {
      let actual =
        DocxRsMarkupNode::InParagraph(vec![ParagraphChild::Run(Default::default())]).to_string();
      assert_eq!(actual, r#"<w:r><w:rPr /></w:r>"#)
    }

    #[test]
    fn test_run_prefix_and_suffix_have_not_changed_when_bumping_docs_rs_version() {
      let actual =
        DocxRsMarkupNode::InRun(vec![RunChild::Break(Break::new(BreakType::Page))]).to_string();
      assert_eq!(actual, r#"<w:br w:type="page" />"#)
    }
  }
}
