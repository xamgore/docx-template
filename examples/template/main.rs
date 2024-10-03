use serde_json::json;
use std::fs::File;
use std::io::BufWriter;

use docx_template::docx_file::DocxFile;
use docx_template::docx_template::DocxTemplate;
use docx_template::transformers::find_and_replace::{Placeholders, Replacements};

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

  let mut template = DocxTemplate {
    template: DocxFile::from_path("./examples/template/input.docx")?,
    patterns: Placeholders::from_json_keys_with_brackets("{", "}", &json),
  };

  let mut output = BufWriter::new(File::create("./examples/template/output.docx").unwrap());
  template.render(&mut output, Replacements::from_json(&json))?;

  Ok(())
}
