use crate::DocxTemplateError;
use std::io;
use thiserror::Error;

pub mod erase_commented;
pub mod find_and_replace;

#[derive(Error, Debug)]
pub enum TransformerError {
  #[error(transparent)]
  WriteIoErr(#[from] io::Error),
  #[error(transparent)]
  ReadXmlErr(#[from] quick_xml::Error),
}

impl From<TransformerError> for DocxTemplateError {
  fn from(value: TransformerError) -> Self {
    match value {
      // probably malformed .docx file
      TransformerError::ReadXmlErr(err) => Self::from(err),
      // as data is written directly to a zip archive, it's a ZipError
      TransformerError::WriteIoErr(err) => Self::ZipErr(err.into()),
    }
  }
}
