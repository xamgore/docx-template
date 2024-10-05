use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};
use std::path::Path;

use serde::Serialize;
use thiserror::Error;
use zip::result::ZipError;
use zip::ZipArchive;

use crate::transformers::find_and_replace::{Placeholders, Replacements};
use crate::{DocxTemplate, DocxTemplateError};

#[derive(Error, Debug)]
pub enum DocxRenderError {
  #[error(transparent)]
  SerdeJsonErr(#[from] serde_json::Error),
  #[error(transparent)]
  DocxTemplateErr(#[from] DocxTemplateError),
}

/// Docx file is essentially a zip archive containing .xml files and images.
#[derive(Debug, Clone)]
pub struct DocxFile<R> {
  pub(crate) archive: ZipArchive<R>,
  // todo: field with a decryption password
}

impl<R> DocxFile<R> {
  pub fn into_template(
    self,
    data: impl Serialize,
  ) -> Result<DocxTemplate<'static, R>, serde_json::Error> {
    self.into_template_having_brackets("{", "}", data)
  }

  pub fn into_template_having_brackets(
    self,
    open_bracket: &str,
    close_bracket: &str,
    data: impl Serialize,
  ) -> Result<DocxTemplate<'static, R>, serde_json::Error> {
    let data = serde_json::to_value(data)?;
    let placeholders =
      Placeholders::from_json_keys_with_brackets(open_bracket, close_bracket, &data);
    let replacements = Replacements::try_from_serializable(&data)?;
    Ok(DocxTemplate::new(self, placeholders, replacements))
  }
}

impl DocxFile<()> {
  pub fn from_slice<A: AsRef<[u8]>>(input: A) -> Result<DocxFile<Cursor<A>>, ZipError> {
    DocxFile::from_reader(Cursor::new(input))
  }

  pub fn from_path<P: AsRef<Path>>(path: P) -> Result<DocxFile<BufReader<File>>, ZipError> {
    DocxFile::from_reader(BufReader::new(File::open(path)?))
  }
}

impl<R: Read + Seek> DocxFile<R> {
  pub fn from_reader(reader: R) -> Result<DocxFile<R>, ZipError> {
    Ok(Self::from_zip_archive(ZipArchive::new(reader)?))
  }

  pub fn from_zip_archive(archive: ZipArchive<R>) -> DocxFile<R> {
    DocxFile { archive }
  }
}
