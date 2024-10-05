use std::fs::File;
use std::io::BufWriter;

use docx_template::{DocxFile, DocxTemplate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let path = "./examples/image-replacement/input.docx";
  let file = DocxFile::from_path(path)?;

  let path = "./examples/image-replacement/output.docx";
  let result = BufWriter::new(File::create(path).unwrap());

  DocxTemplate::new_with_placeholders(file, Default::default())
    .replace_inner_file("word/media/image1.jpg", include_bytes!("cat.jpg"))
    .render_to(result)?;

  Ok(())
}
