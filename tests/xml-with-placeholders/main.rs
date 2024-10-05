use std::io::Cursor;

use insta::assert_snapshot;

use docx_template::DocxTemplateError;
use docx_template::{FindAndReplace, Placeholders, Replacements};

#[test]
fn test() -> Result<(), Box<dyn std::error::Error>> {
  let template = include_bytes!("input.xml");
  
  let placeholders = Placeholders::from_iter_with_brackets(
    "{",
    "}",
    ["single", "fragmented_placeholder", "foo_bar", "some_placeholder"],
  );

  let replacements = Replacements::from_iter(["[1]", "[2]", "[3]", "[4]", "[5]"]);

  let result = FindAndReplace { placeholders, replacements }
    .transform_stream(template, Cursor::new(Vec::new()))
    .map_err(DocxTemplateError::from)?
    .into_inner();

  let xml = std::str::from_utf8(&result)?;
  assert_snapshot!(xml);
  Ok(())
}
