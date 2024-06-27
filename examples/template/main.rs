use serde_json::json;
use std::fs::File;
use std::io::BufWriter;

use docx_template::docx_file::DocxFile;
use docx_template::docx_template::DocxTemplate;
use docx_template::transformers::find_and_replace::{Patterns, Replacement};

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

  let path = "./examples/template/input.docx";
  let mut template = DocxTemplate {
    template: DocxFile::from_path(path)?,
    patterns: Patterns::from_json_with_brackets("{", "}", &json),
  };

  let path = "./examples/template/output.docx";
  let mut result = BufWriter::new(File::create(path).unwrap());
  let replacements: Vec<_> =
    json.as_object().unwrap().values().cloned().map(Replacement::from).collect();
  template.render(&mut result, &replacements)?;

  Ok(())
}
