use thiserror::Error;
use std::io;

pub mod find_and_replace;
pub(crate) mod erase_commented;

#[derive(Error, Debug)]
pub enum TransformerError {
  #[error(transparent)]
  WriteIoErr(#[from] io::Error),
  #[error(transparent)]
  ReadXmlErr(#[from] quick_xml::Error),
}
