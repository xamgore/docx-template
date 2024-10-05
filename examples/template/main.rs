use serde_json::json;
use std::fs::File;
use std::io::BufWriter;

use docx_template::DocxFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let json = json!({
    "key":                         "REPLACE some more",
    "key-with-dash":               "REPLACE",
    "key-with-dashes":             "REPLACE",
    "key with space":              "REPLACE",
    "key_with_underscore":         "REPLACE",
    "multiline":                   "REPLACE",
    "key.with.dots":               "REPLACE",
    "mixed-key.separator_styles#": "REPLACE",
    "yet-another_placeholder":     "REPLACE",
    "foo":                         "bar",
  });

  let output = BufWriter::new(File::create("./examples/template/output.docx").unwrap());

  DocxFile::from_path("./examples/template/input.docx")?
    .into_template_having_brackets("{", "}", json)?
    .render_to(output)?;

  Ok(())
}
