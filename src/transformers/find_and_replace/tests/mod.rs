use super::*;

fn run<const T: usize>(subs: [(&str, &str); T], input: &str) -> String {
  let replacements = subs.map(|(_, s)| s).map(Value::from_text);
  let buf = FindAndReplace {
    placeholders: Placeholders::from_iter(subs.into_iter().map(|(pattern, _)| pattern)),
    replacements: Replacements::from_slice(replacements.as_slice()),
  }
  .transform_stream(input, Vec::new())
  .unwrap();
  String::from_utf8(buf).unwrap()
}

mod replacements {
  use super::*;

  #[test]
  fn leaves_text_without_holes_alone() {
    insta::assert_snapshot!(
      run(
        [("{hole}", "[]")],
        r#"<w:p><w:r><w:t>just text</w:t></w:r></w:p>"#,
      ),
      @r###"<w:p><w:r><w:t>just text</w:t></w:r></w:p>"###,
    );
  }

  #[test]
  fn modifies_placeholder() {
    insta::assert_snapshot!(
      run(
        [("{hole}", "[]")],
        r#"<w:p><w:r><w:t>{hole}</w:t></w:r></w:p>"#,
      ),
      @r###"<w:p><w:r><w:t>[]</w:t></w:r></w:p>"###,
    );
  }

  #[test]
  fn is_simultaneous() {
    insta::assert_snapshot!(
      run(
        [("{hole}", "[]"), ("[]", "a bug")],
        r#"<w:p><w:r><w:t>{hole}</w:t></w:r></w:p>"#,
      ),
      @r###"<w:p><w:r><w:t>[]</w:t></w:r></w:p>"###,
    );
  }
}

mod structural {
  use indoc::indoc;

  use super::*;

  #[test]
  fn single_hole_with_prefix() {
    insta::assert_snapshot!(
      run(
        [("{hole}", "[]")],
        r#"<w:p><w:r><w:t>text{hole}</w:t></w:r></w:p>"#,
      ),
      @r###"<w:p><w:r><w:t>text[]</w:t></w:r></w:p>"###,
    );
  }

  #[test]
  fn single_hole_with_suffix() {
    insta::assert_snapshot!(
      run(
        [("{hole}", "[]")],
        r#"<w:p><w:r><w:t>{hole}text</w:t></w:r></w:p>"#,
      ),
      @r###"<w:p><w:r><w:t>[]text</w:t></w:r></w:p>"###,
    );
  }

  #[test]
  fn multiple_holes() {
    insta::assert_snapshot!(
      run(
        [("{hole}", "[]")],
        r#"<w:p><w:r><w:t>{hole}{hole}</w:t></w:r></w:p>"#,
      ),
      @r###"<w:p><w:r><w:t>[][]</w:t></w:r></w:p>"###,
    );
  }

  #[test]
  fn multiple_holes_with_text_between() {
    insta::assert_snapshot!(
      run(
        [("{hole}", "[]")],
        r#"<w:p><w:r><w:t>{hole}text{hole}</w:t></w:r></w:p>"#,
      ),
      @r###"<w:p><w:r><w:t>[]text[]</w:t></w:r></w:p>"###,
    );
  }

  #[test]
  fn comment_with_hole() {
    insta::assert_snapshot!(
      run(
        [("{hole}", "[]")],
        r#"<w:p><w:r><!-- something other than a <w:t> with a {hole} --></w:r></w:p>"#,
      ),
      @r###"<w:p><w:r><!-- something other than a <w:t> with a {hole} --></w:r></w:p>"###,
    );
  }

  #[test]
  fn split_placeholder_in_text_tags() {
    insta::assert_snapshot!(
      run(
        [("{hole}", "[]")],
        indoc! {r#"
          <w:p>
            <w:r>
              <w:t>{</w:t>
              <w:t>hole</w:t>
              <w:t>}</w:t>
            </w:r>
          </w:p>
        "#},
      ),
      @r###"
        <w:p>
          <w:r>
            <w:t></w:t>
            <w:t></w:t>
            <w:t>[]</w:t>
          </w:r>
        </w:p>
      "###,
    );
  }

  #[test]
  fn split_placeholder() {
    insta::assert_snapshot!(
      run(
        [("{hole}", "[]")],
        indoc! {r#"
          <w:p>
            <w:r>
              <w:t>text{</w:t>
              <w:t>hole</w:t>
              <w:t>}text</w:t>
            </w:r>
          </w:p>
        "#},
      ),
      @r###"
        <w:p>
          <w:r>
            <w:t>text</w:t>
            <w:t></w:t>
            <w:t>[]text</w:t>
          </w:r>
        </w:p>
      "###,
    );
  }

  #[test]
  fn split_placeholder_in_run_tags1() {
    insta::assert_snapshot!(
      run(
        [("{hole}", "[]")],
        indoc! {r#"
          <w:p>
            <w:r><w:t>{</w:t></w:r>
            <w:r><w:t>hole</w:t></w:r>
            <w:r><w:t>}</w:t></w:r>
          </w:p>
        "#},
      ),
      @r###"
        <w:p>
          <w:r><w:t></w:t></w:r>
          <w:r><w:t></w:t></w:r>
          <w:r><w:t>[]</w:t></w:r>
        </w:p>
      "###,
    );
  }

  #[test]
  fn split_placeholder_in_run_tags2() {
    insta::assert_snapshot!(
      run(
        [("{hole}", "[]")],
        indoc! {r#"
          <w:p>
            <w:r><w:t>number</w:t></w:r>
            <w:r><w:t>{</w:t></w:r>
            <w:r><w:t>hole</w:t></w:r>
            <w:r><w:t>}</w:t></w:r>
          </w:p>
        "#},
      ),
      @r###"
        <w:p>
          <w:r><w:t>number</w:t></w:r>
          <w:r><w:t></w:t></w:r>
          <w:r><w:t></w:t></w:r>
          <w:r><w:t>[]</w:t></w:r>
        </w:p>
      "###,
    );
  }

  #[test]
  fn split_placeholder_in_paragraph_texts() {
    insta::assert_snapshot!(
      run(
        [("{hole}", "[]")],
        indoc! {r#"
          <w:p><w:r><w:t>{</w:t></w:r></w:p>
          <w:p><w:r><w:t>hole</w:t></w:r></w:p>
          <w:p><w:r><w:t>}</w:t></w:r></w:p>
        "#},
      ),
      @r###"
        <w:p><w:r><w:t>{</w:t></w:r></w:p>
        <w:p><w:r><w:t>hole</w:t></w:r></w:p>
        <w:p><w:r><w:t>}</w:t></w:r></w:p>
      "###,
    );
  }

  #[test]
  fn multiple_paragraphs() {
    insta::assert_snapshot!(
      run(
        [("{hole}", "[]")],
        indoc! {r#"
          <w:p><w:r><w:t>a{hole}a</w:t></w:r></w:p>
          <w:p><w:r><w:t>a{hole}a</w:t></w:r></w:p>
        "#},
      ),
      @r###"
        <w:p><w:r><w:t>a[]a</w:t></w:r></w:p>
        <w:p><w:r><w:t>a[]a</w:t></w:r></w:p>
      "###,
    );
  }
}
