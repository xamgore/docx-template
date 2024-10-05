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

impl DocxPartType {
  pub fn comments() -> &'static str {
    "word/comments.xml"
  }
}

impl<S: AsRef<str>> From<S> for DocxPartType {
  fn from(path: S) -> Self {
    match path.as_ref() {
      "word/document.xml" => Self::Main,
      "word/comments.xml" => Self::Comments,
      path if regex!(r#"word/header[0-9]*.xml"#).is_match(path) => Self::Header,
      path if regex!(r#"word/footer[0-9]*.xml"#).is_match(path) => Self::Footer,
      _ => Self::Unknown,
    }
  }
}
