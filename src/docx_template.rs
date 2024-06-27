use std::collections::HashMap;
use std::io;

use aho_corasick::BuildError;
use hard_xml::XmlWrite;
use thiserror::Error;
use zip::result::ZipError;

use crate::docx_file::DocxFile;
use crate::docx_part::DocxPartType;
use crate::transformers::find_and_replace::{FindAndReplaceTransformer, Patterns, Replacement};
use crate::transformers::TransformerError;
use crate::zip_file_ext::ZipFileExt;

pub struct DocxTemplate<R> {
  pub patterns: Patterns,
  pub template: DocxFile<R>,
}

#[derive(Error, Debug)]
pub enum DocxTemplateError {
  #[error(transparent)]
  ZipErr(#[from] ZipError),
  #[error(transparent)]
  AutomatonBuildErr(#[from] BuildError),
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

impl<R: io::Read + io::Seek> DocxTemplate<R> {
  pub fn render<W: io::Write + io::Seek>(
    &mut self,
    writer: W,
    replacements: &[Replacement],
  ) -> Result<W, DocxTemplateError> {
    let mut result = zip::ZipWriter::new(writer);

    let _comments = self.extract_comments();

    // TODO: The code below is just a bad written visitor pattern,
    //       where we stream each file to the TemplateEngine transformer.
    //       Instead we could have a list of transformers, with extra functionality,
    //       like switching checkboxes (on/off), generating bullet lists, etc.

    for idx in 0..self.template.archive.len() {
      let mut part = self.template.archive.by_index(idx)?;

      match part.name().parse().unwrap_or_default() {
        DocxPartType::Comments | DocxPartType::Unknown => {
          // copy-paste compressed bytes directly to the resulting archive
          result.raw_copy_file(part)?
        }
        DocxPartType::Main | DocxPartType::Header | DocxPartType::Footer => {
          // declare a file
          result.start_file(part.name(), part.to_options())?;

          // read the file into a buffer, transform it, write to the resulting archive
          let mut buf = String::new();
          io::Read::read_to_string(&mut part, &mut buf).map_err(ZipError::Io)?;

          FindAndReplaceTransformer { patterns: self.patterns.clone(), replacements }
            .transform_stream(buf.as_bytes(), &mut result)
            .map_err(DocxTemplateError::from)?;
        }
      }
    }

    Ok(result.finish()?)
  }

  fn extract_comments(&mut self) -> HashMap<String, isize> {
    let mut part = match self.template.archive.by_name(DocxPartType::comments()) {
      Ok(part) => part,
      Err(_) => return Default::default(),
    };

    let mut buf = String::with_capacity(part.size() as usize);
    io::Read::read_to_string(&mut part, &mut buf).ok();

    use hard_xml::XmlRead;
    let Ok(def) = docx_rust::document::Comments::from_str(&buf) else { panic!() };

    def
      .comments
      .into_iter()
      .filter_map(|com| com.content.to_string().ok().zip(com.id))
      .collect::<HashMap<_, _>>()
  }
}
