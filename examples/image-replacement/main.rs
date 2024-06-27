use std::fs::File;
use std::io::BufWriter;

use docx_template::docx_file::DocxFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let path = "./examples/image-replacement/input.docx";
  let mut file = DocxFile::from_path(path)?;

  let path = "./examples/image-replacement/output.docx";
  let mut result = BufWriter::new(File::create(path).unwrap());

  let cat_img = include_bytes!("cat.jpg");
  file.replace_file(&mut result, "word/media/image1.jpg", cat_img)?;

  Ok(())
}
