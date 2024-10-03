use crate::transformers::find_and_replace::{Placeholders, Replacements};
use crate::zip_file_ext::ZipFileExt;
use crate::{DocxTemplate, DocxTemplateError};
use serde::Serialize;
use std::fs::File;
use std::io;
use std::path::Path;
use thiserror::Error;
use zip::result::ZipError;
use zip::ZipArchive;

/// Docx file is essentially a zip archive containing .xml files and images.
///
pub struct DocxFile<R> {
  pub(crate) archive: ZipArchive<R>,
  // todo: field with a decryption password
}

impl DocxFile<()> {
  pub fn from_slice<A: AsRef<[u8]>>(input: A) -> Result<DocxFile<io::Cursor<A>>, ZipError> {
    DocxFile::from_reader(io::Cursor::new(input))
  }

  pub fn from_path<P: AsRef<Path>>(path: P) -> Result<DocxFile<io::BufReader<File>>, ZipError> {
    DocxFile::from_reader(io::BufReader::new(File::open(path)?))
  }
}

impl<R: io::Read + io::Seek> DocxFile<R> {
  pub fn from_reader(reader: R) -> Result<DocxFile<R>, ZipError> {
    Ok(Self::from_zip_archive(ZipArchive::new(reader)?))
  }

  pub fn from_zip_archive(archive: ZipArchive<R>) -> DocxFile<R> {
    DocxFile { archive }
  }

  pub fn render_with_brackets<W: io::Write + io::Seek, S: Serialize>(
    self,
    open_bracket: &str,
    close_bracket: &str,
    data: S,
    mut writer: W,
  ) -> Result<(), DocxRenderError> {
    let data = serde_json::to_value(data)?;

    let mut template = DocxTemplate {
      template: self,
      patterns: Placeholders::from_json_keys_with_brackets(open_bracket, close_bracket, &data),
    };

    template.render(&mut writer, Replacements::from_json(&data))?;
    Ok(())
  }

  pub fn replace_file<W: io::Write + io::Seek>(
    &mut self,
    writer: W,
    filename: &str,
    content: &[u8],
  ) -> Result<W, DocxTemplateError> {
    let mut result = zip::ZipWriter::new(writer);

    for idx in 0..self.archive.len() {
      let part = self.archive.by_index(idx)?;

      if (part.name()) == filename {
        // declare a file
        result.start_file(part.name(), part.to_options())?;
        // pipe passed bytes
        io::Write::write_all(&mut result, content).map_err(ZipError::Io)?;
      } else {
        // copy-paste compressed bytes directly to the resulting archive
        result.raw_copy_file(part)?
      }
    }

    Ok(result.finish()?)
  }
}

#[derive(Error, Debug)]
pub enum DocxRenderError {
  #[error(transparent)]
  SerdeJsonErr(#[from] serde_json::Error),
  #[error(transparent)]
  DocxTemplateErr(#[from] DocxTemplateError),
}
