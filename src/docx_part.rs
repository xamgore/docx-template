/// Docx is an archive which contains a lot of XML files.
/// Different parts of layouts are stored in own files to reduce duplication.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DocxPartType {
  Main,
  Header,
  Footer,
  Comments,
  #[default]
  Unknown,
}

#[allow(missing_docs)]
impl DocxPartType {
  #[allow(dead_code)]
  pub fn comments() -> &'static str {
    "word/comments.xml"
  }
}

impl<S: AsRef<str>> From<S> for DocxPartType {
  fn from(path: S) -> Self {
    match path.as_ref() {
      "word/document.xml" => Self::Main,
      "word/comments.xml" => Self::Comments,
      // it's more like "word/header[0-9]*.xml", but regex crate is too heavy here
      path if path.starts_with(r#"word/header"#) && path.ends_with(".xml") => Self::Header,
      path if path.starts_with(r#"word/footer"#) && path.ends_with(".xml") => Self::Footer,
      _ => Self::Unknown,
    }
  }
}
