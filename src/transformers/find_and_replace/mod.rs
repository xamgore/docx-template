use std::io;
use std::ops::Range;

use aho_corasick::Anchored;
use quick_xml::events::Event;
use crate::transformers::TransformerError;

pub use self::patterns::Patterns;
pub use self::replacement::Replacement;

mod patterns;
mod replacement;

// TODO: Consider implementing `io::Read` to enable dynamic placeholder replacement during reading.
//       It would simplify transformer down to its variant ReadXmlError.
//
//       let reader = quick_xml::Reader::from_reader(BufReader::new(file));
//       dbg!(reader.read_event_into(&mut buf));

pub struct FindAndReplaceTransformer<'r> {
  pub patterns: Patterns,
  pub replacements: &'r [Replacement<'r>],
}

impl<'subs> FindAndReplaceTransformer<'subs> {
  pub fn transform_stream<In: AsRef<[u8]>, Out: io::Write>(
    &self,
    input: In,
    mut output: Out,
  ) -> Result<Out, TransformerError> {
    let mut reader = quick_xml::Reader::from_reader(input.as_ref());
    reader.config_mut().check_end_names = true;

    let mut text_spans = Vec::<Range<usize>>::new();
    let mut in_paragraph = false;
    let mut in_run = false;

    // the absolute position over the entire stream
    let mut reported = 0;

    loop {
      match reader.read_event()? {
        Event::Start(tag) => match tag.name().as_ref() {
          b"w:p" => in_paragraph = true,
          b"w:r" => in_run = true,
          b"w:t" if in_paragraph && in_run => {
            let span = reader.read_to_end(tag.name())?;
            text_spans.push(span.start as usize..span.end as usize);
          }
          _ => {}
        },
        Event::End(tag) => match tag.name().as_ref() {
          b"w:p" => {
            reported = self.transform_paragraph(&input, &mut output, &text_spans, reported)?;
            text_spans.clear();
            in_paragraph = false;
          }
          b"w:r" => in_run = false,
          _ => {}
        },
        Event::Eof => break,
        _ => {
          // needs no action, as the content besides <w:p> tags is copied elsewhere
        }
      }
    }

    // return the tail
    output.write_all(&input.as_ref()[reported..])?;
    Ok(output)
  }

  pub fn transform_paragraph<In: AsRef<[u8]>, Out: io::Write>(
    &self,
    input: In,
    out: &mut Out,
    spans: &[Range<usize>],
    mut reported: usize,
  ) -> io::Result<usize> {
    let Ok(start) = self.patterns.automaton.start_state(Anchored::No) else {
      unreachable!("aho-corasick automaton misconfiguration");
    };
    let mut sid = start;

    for (span_idx, span) in spans.iter().enumerate() {
      // todo: quick_xml::Decoder::decode(&input[span])
      // don't decode for now, assume it's utf8
      // todo: encode replacements — should probably be done on the lower io level

      // span's space offset
      for (offset, byte) in input.as_ref()[span.clone()].iter().copied().enumerate() {
        sid = self.patterns.automaton.next_state(Anchored::No, sid, byte);
        if !self.patterns.automaton.is_match(sid) {
          continue;
        }

        let pat_id = self.patterns.automaton.match_pattern(sid, 0);
        let pat_len = self.patterns.automaton.pattern_len(pat_id);
        sid = start;

        let r#match = Range { start: (offset + 1).saturating_sub(pat_len), end: offset + 1 };

        // if the match is split between K spans, let's go backwards and find the 1st span
        if r#match.len() < pat_len {
          let (mut idx, mut bytes_to_consume) = (span_idx, pat_len - r#match.len());
          loop {
            idx -= 1;
            if bytes_to_consume > spans[idx].len() {
              bytes_to_consume -= spans[idx].len();
            } else {
              break;
            }
          }

          let (first_span_idx, tail_len) = (idx, bytes_to_consume);

          for (idx, span) in spans[first_span_idx..span_idx].iter().enumerate() {
            // for the 1st span we output the internal text as is, excluding the tail
            // for the 2nd, 3rd, ..., (K-1)-th spans we omit the internal text
            let until = if idx == 0 { span.end - tail_len } else { span.start };
            out.write_all(&input.as_ref()[reported..until])?;
            reported = span.end;
          }
        }

        // for K-th span we put the replacement instead of the match
        out.write_all(&input.as_ref()[reported..(span.start + r#match.start)])?;

        let replacement = match &self.replacements[pat_id.as_usize()] {
          Replacement::Xml(xml) => xml.as_bytes(),
        };
        out.write_all(replacement)?;
        reported = span.start + r#match.end;
      }
    }

    Ok(reported)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn run<const T: usize>(subs: [(&str, &str); T], input: &str) -> String {
    let patterns = Patterns::from_iter(subs.into_iter().map(|(pattern, _)| pattern));
    let replacements = subs.map(|(_, s)| s).map(Replacement::from_text);
    let buf = FindAndReplaceTransformer { patterns, replacements: replacements.as_slice() }
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
}
