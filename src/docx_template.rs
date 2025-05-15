use aho_corasick::BuildError;
use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read, Seek, Write};
use thiserror::Error;
use zip::read::ZipFile;
use zip::result::ZipError;

use crate::docx_file::DocxFile;
use crate::docx_part::DocxPartType;
use crate::transformers::find_and_replace::{FindAndReplace, Placeholders, Replacements};
use crate::zip_file_ext::ZipFileExt;

#[derive(Debug, Clone)]
/// Builder accumulating all the transformations over `.docx` file.
pub struct DocxTemplate<'a, R> {
  file: DocxFile<R>,
  placeholders: Placeholders,
  replacements: Option<Replacements<'a>>,
  inner_files_to_replace: HashMap<&'a str, &'a [u8]>,
  comments_to_delete: HashSet<&'a str>,
}

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum CantRenderError {
  /// Read/write issues with the zip archive.
  #[error(transparent)]
  ZipErr(#[from] ZipError),
  /// Could not compile placeholders into an automaton.
  #[error(transparent)]
  AutomatonBuildErr(#[from] BuildError),
  /// Probably a malformed `.docx` file.
  #[error(transparent)]
  ReadXmlErr(#[from] quick_xml::Error),
}

impl<'a, R> DocxTemplate<'a, R> {
  /// Create a template to be rendered once.
  ///
  /// Placeholders and replacements must have the same length.
  pub fn new(
    file: DocxFile<R>,
    placeholders: Placeholders,
    replacements: Replacements<'a>,
  ) -> Self {
    debug_assert_eq!(placeholders.len(), replacements.len());
    Self {
      file,
      placeholders,
      replacements: Some(replacements),
      inner_files_to_replace: Default::default(),
      comments_to_delete: Default::default(),
    }
  }
}

impl<R> DocxTemplate<'_, R> {
  /// Create a template to be rendered multiple times.
  ///
  /// In such scenarios it's wise to convert placeholders to an automaton once,
  /// then reuse it between render calls. Paired with [DocxTemplate::replace_placeholders_with].
  ///
  /// ```rust
  /// # use docx_template::{DocxFile, DocxTemplate, Placeholders, Replacements};
  ///
  /// let mut template = DocxTemplate::new_with_placeholders(
  ///   DocxFile::from_path("examples/template/input.docx")?,
  ///   Placeholders::from_iter(["{name}", "{address}"]),
  /// );
  ///
  /// std::fs::write(
  ///   "output.docx",
  ///   template
  ///     .replace_placeholders_with(Replacements::from_iter([
  ///       "Alphabet Inc.",
  ///       "1600 Amphitheatre Parkway in Mountain View, California",
  ///     ]))
  ///     .render()?,
  /// )?;
  ///
  /// # Ok::<(), Box<dyn std::error::Error>>(())
  /// ```
  pub fn new_with_placeholders(file: DocxFile<R>, placeholders: Placeholders) -> Self {
    Self {
      file,
      placeholders,
      replacements: None,
      inner_files_to_replace: Default::default(),
      comments_to_delete: Default::default(),
    }
  }
}

impl<'a, R: Read + Seek> DocxTemplate<'a, R> {
  /// Set values to be used instead of placeholders.
  ///
  /// Placeholders and replacements must have the same length.
  ///
  /// Usually paired with [DocxTemplate::new_with_placeholders].
  pub fn replace_placeholders_with(&mut self, replacements: Replacements<'a>) -> &mut Self {
    debug_assert_eq!(self.placeholders.len(), replacements.len());
    self.replacements = Some(replacements);
    self
  }

  #[doc(hidden)]
  // todo: a screenshot
  /// Encountering a comment with `{placeholder}` content will _delete_ the whole commented block.
  ///
  /// To remove the image from a document, wrap it in a comment, and delete using this method.
  ///
  /// ```rust
  /// # use std::fs::File;
  /// # use std::error::Error;
  /// # use std::io::{Read, Seek, Cursor};
  /// # use docx_template::{DocxTemplate, DocxFile, CantRenderError};
  ///
  /// # fn generate<R>(template: &mut DocxTemplate<R>) -> Result<(), Box<dyn Error>> where R: Read + Seek {
  /// template
  ///   .remove_commented_block("{image1}")
  ///   .render_to(File::create("output.docx")?)?;
  /// # Ok(())
  /// # }
  /// ```
  pub fn remove_commented_block(&mut self, placeholder: &'a str) -> &mut Self {
    self.comments_to_delete.insert(placeholder);
    self
  }

  /// Replace a file inside `.docx` archive.
  ///
  /// Method is quite handy for switching images in a document.
  /// The formatting options won't be changed, so it's wise to preserve
  /// the image's size (width × height), as well as the codec (png, jpg, ...).
  ///
  /// ```rust
  /// # use std::fs::File;
  /// # use std::io::{Read, Seek};
  /// # use docx_template::{DocxTemplate, DocxFile, CantRenderError};
  ///
  /// # fn generate<R>(template: &mut DocxTemplate<R>) -> Result<(), CantRenderError> where R: Read + Seek {
  /// let cat = include_bytes!("../examples/image-replacement/cat.jpg");
  /// template.replace_inner_file("word/media/image1.jpg", cat).render()?;
  /// # Ok(())
  /// # }
  /// ```
  pub fn replace_inner_file(&mut self, inner_path: &'a str, bytes: &'a [u8]) -> &mut Self {
    self.inner_files_to_replace.insert(inner_path, bytes);
    self
  }
}

impl<R: Read + Seek> DocxTemplate<'_, R> {
  /// Render the template applying all the transformations set before.
  ///
  /// Returns a byte array, content of a `.docx` file.
  pub fn render(&mut self) -> Result<Vec<u8>, CantRenderError> {
    self.render_to(Cursor::new(Vec::new())).map(Cursor::into_inner)
  }

  /// Render the template applying all the transformations set before.
  ///
  /// Writes the resulting `.docx` bytes to the `writer` stream.
  ///
  /// Even though the function accepts generic parameter writer: `W` by value,
  /// you [may pass] a `&mut writer` reference if necessary.
  ///
  /// [may pass]: https://rust-lang.github.io/api-guidelines/interoperability.html#generic-readerwriter-functions-take-r-read-and-w-write-by-value-c-rw-value
  pub fn render_to<W: Write + Seek>(&mut self, writer: W) -> Result<W, CantRenderError> {
    let mut result = zip::ZipWriter::new(writer);

    let find_and_replace = self
      .replacements
      .clone()
      .map(|replacements| FindAndReplace { placeholders: self.placeholders.clone(), replacements });

    // let _comments = self._extract_comments();

    for idx in 0..self.file.archive.len() {
      let mut f: ZipFile<R> = self.file.archive.by_index(idx)?;

      if let Some(&buffer) = self.inner_files_to_replace.get(f.name()) {
        // declare a file
        result.start_file(f.name(), f.to_options())?;
        // pipe passed bytes
        Write::write_all(&mut result, buffer).map_err(ZipError::Io)?;

        continue;
      }

      let part_of_layout: DocxPartType = f.name().into();

      match part_of_layout {
        DocxPartType::Comments | DocxPartType::Unknown => {
          // copy-paste compressed bytes directly to the resulting archive
          result.raw_copy_file(f)?
        }
        DocxPartType::Main | DocxPartType::Header | DocxPartType::Footer => {
          if let Some(ref find_and_replace) = find_and_replace {
            // declare a file
            result.start_file(f.name(), f.to_options())?;

            // read the file into a buffer, transform it, write to the resulting archive
            let mut buf = String::new();
            Read::read_to_string(&mut f, &mut buf).map_err(ZipError::Io)?;

            find_and_replace
              .transform_stream(buf.as_bytes(), &mut result)
              .map_err(CantRenderError::from)?;
          } else {
            result.raw_copy_file(f)?;
          }
        }
      }
    }

    Ok(result.finish()?)
  }

  #[cfg(feature = "docx-rust")]
  fn _extract_comments(&mut self) -> HashMap<String, isize> {
    let mut part = match self.file.archive.by_name(DocxPartType::comments()) {
      Ok(part) => part,
      Err(_) => return Default::default(),
    };

    let mut buf = String::with_capacity(part.size() as usize);
    Read::read_to_string(&mut part, &mut buf).ok();

    use hard_xml::{XmlRead, XmlWrite};
    let Ok(def) = docx_rust::document::Comments::from_str(&buf) else { panic!() };

    def
      .comments
      .into_iter()
      .filter_map(|com| com.content.to_string().ok().zip(com.id))
      .collect::<HashMap<_, _>>()
  }
}
