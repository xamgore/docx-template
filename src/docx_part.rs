use std::str::FromStr;

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

impl FromStr for DocxPartType {
  type Err = std::convert::Infallible;

  fn from_str(path: &str) -> Result<Self, Self::Err> {
    Ok(match path {
      "word/document.xml" => Self::Main,
      "word/comments.xml" => Self::Comments,
      path if regex!(r#"word/header[0-9]*.xml"#).is_match(path) => Self::Header,
      path if regex!(r#"word/footer[0-9]*.xml"#).is_match(path) => Self::Footer,
      _ => Self::Unknown,
    })
  }
}
