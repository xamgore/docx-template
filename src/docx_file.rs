use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

use serde::Serialize;
use zip::result::ZipError;
use zip::ZipArchive;

use crate::transformers::find_and_replace::{Placeholders, Replacements};
use crate::{CantSerializeError, DocxTemplate};

/// Docx file is essentially a zip archive containing .xml files and images.
#[derive(Debug, Clone)]
pub struct DocxFile<R> {
  pub(crate) archive: ZipArchive<R>,
  // todo: field with a decryption password
}

impl DocxFile<()> {
  /// A shortcut to read a `.docx` file by its path.
  ///
  /// It opens [`io::File`](std::io::File) and wrap it in a [`BufReader`](std::io::BufReader) to reduce the total number of system calls.
  ///
  /// # Errors
  ///
  /// This function will return an error if `path` does not already exist,
  /// or the document is a malformed zip archive.
  pub fn from_path<P: AsRef<Path>>(path: P) -> Result<DocxFile<BufReader<File>>, ZipError> {
    DocxFile::from_reader(BufReader::new(File::open(path)?))
  }

  /// Read a `.docx` file from a reader.
  ///
  /// [`Seek`](std::io::Seek) trait is required, as any `.docx` file is essentially a `.zip` archive,
  /// thus the inner files might be read in any order.
  ///
  /// In-memory stored documents presented as byte arrays compose well with [`Cursor`](std::io::Cursor).
  ///
  /// ```rust
  /// # use docx_template::DocxFile;
  /// # use std::io::Cursor;
  /// let data: &[u8] = include_bytes!("../examples/template/input.docx");
  /// DocxFile::from_reader(Cursor::new(data))
  /// # .unwrap();
  /// ```
  ///
  /// Even though the function accepts generic parameter reader: `R` by value,
  /// you [may pass] a `&mut reader` reference if necessary.
  ///
  /// [may pass]: https://rust-lang.github.io/api-guidelines/interoperability.html#generic-readerwriter-functions-take-r-read-and-w-write-by-value-c-rw-value
  ///
  /// # Errors
  ///
  /// This function will return an error if the document is a malformed zip archive.
  pub fn from_reader<R: Read + Seek>(reader: R) -> Result<DocxFile<R>, ZipError> {
    Ok(Self::from_zip_archive(ZipArchive::new(reader)?))
  }

  /// Read a `.docx` file from a `.zip` archive.
  pub fn from_zip_archive<R: Read + Seek>(archive: ZipArchive<R>) -> DocxFile<R> {
    DocxFile { archive }
  }
}

impl<R> DocxFile<R> {
  /// A shortcut method for converting the `.docx` file into a template having `{placeholders}`.
  ///
  /// Placeholders and replacements will be taken from the `data` argument during render stage.
  ///
  /// ```rust
  /// # use std::fs::File;
  /// # use std::io::BufWriter;
  /// # use docx_template::DocxFile;
  ///
  /// #[derive(serde::Serialize)]
  /// struct Data { id: u64 }
  ///
  /// DocxFile::from_path("examples/template/input.docx")?
  ///   .into_template(Data { id: 42 })?
  ///   .render_to(BufWriter::new(File::create("out.docx")?))?;
  ///
  /// # Ok::<(), Box<dyn std::error::Error>>(())
  /// ```
  pub fn into_template(
    self,
    data: impl Serialize,
  ) -> Result<DocxTemplate<'static, R>, CantSerializeError> {
    self.into_template_having_brackets("{", "}", data)
  }

  /// A shortcut method for converting the `.docx` file into a template.
  /// Opening and closing brackets are defined through arguments.
  ///
  /// Placeholders and replacements will be taken from the `data` argument during render stage.
  ///
  /// ```rust
  /// # use std::fs::File;
  /// # use std::io::BufWriter;
  /// # use docx_template::DocxFile;
  ///
  /// #[derive(serde::Serialize)]
  /// struct Data { id: u64 }
  ///
  /// DocxFile::from_path("examples/template/input.docx")?
  ///   .into_template_having_brackets("{{", "}}", Data { id: 42 })?
  ///   .render_to(BufWriter::new(File::create("out.docx")?))?;
  ///
  /// # Ok::<(), Box<dyn std::error::Error>>(())
  /// ```
  pub fn into_template_having_brackets(
    self,
    open_bracket: &str,
    close_bracket: &str,
    data: impl Serialize,
  ) -> Result<DocxTemplate<'static, R>, CantSerializeError> {
    let data = serde_json::to_value(data)?;
    let placeholders =
      Placeholders::from_json_keys_with_brackets(open_bracket, close_bracket, &data);
    let replacements = Replacements::from_json_object_fields(&data);
    Ok(DocxTemplate::new(self, placeholders, replacements))
  }
}
