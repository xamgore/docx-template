use std::io::Cursor;

use insta::assert_snapshot;

use docx_template::docx_template::DocxTemplateError;
use docx_template::transformers::find_and_replace::{
  FindAndReplaceTransformer, Patterns, Replacement,
};

#[test]
fn test() -> Result<(), Box<dyn std::error::Error>> {
  let patterns = Patterns::from_iter_with_brackets(
    "{",
    "}",
    ["single", "fragmented_placeholder", "foo_bar", "some_placeholder"],
  );
  let template = include_bytes!("input.xml");

  let replacements = &["[1]", "[2]", "[3]", "[4]", "[5]"].map(Replacement::from);
  let mut result = Cursor::new(Vec::new());

  FindAndReplaceTransformer { patterns, replacements }
    .transform_stream(template, &mut result)
    .map_err(DocxTemplateError::from)?;

  let result = result.into_inner();
  let xml = std::str::from_utf8(&result)?;
  assert_snapshot!(xml);
  Ok(())
}
