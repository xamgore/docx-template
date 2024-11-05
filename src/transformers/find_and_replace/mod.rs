use std::io;
use std::ops::Range;

use crate::transformers::TransformerError;
use aho_corasick::Anchored;
use quick_xml::events::Event;

pub use self::placeholders::Placeholders;
pub use self::replacements::Replacements;
pub use self::value::Value;

mod placeholders;
mod replacements;
mod value;

#[cfg(test)]
mod tests;

// TODO: Consider implementing `io::Read` to enable dynamic placeholder replacement during reading.
//       It would simplify transformer down to its variant ReadXmlError.
//
//       let reader = quick_xml::Reader::from_reader(BufReader::new(file));
//       dbg!(reader.read_event_into(&mut buf));

/// Pipes the input stream to the output stream. If text patterns (placeholders) are matched,
/// they will be replaced according to the `replacements` list.
///
/// A placeholder can reside in more than one `<w:run>`. This transformer replaces the first bit
/// and erases the rest. Each paragraph is transformed individually to prevent odd cases.
///
/// ```xml
/// <w:run>{place</w:run><w:run>holder}</w:run>
/// <w:run>🦀</w:run><w:run></w:run>
/// ```
/// The algorithm reads paragraphs one by one, accumulating a list of text spans met.
/// At the end of each paragraph the text is fed to an automaton, which halts when
/// any of the patterns was read. Then series of XML markups from between the placeholder's bits
/// like `</w:run><w:run>` are pushed to the output stream, interspersed with either
/// the corresponding `replacement` string (first bit), or an empty string (the rest bits).
#[derive(Debug, Default, Clone)]
pub struct FindAndReplace<'r> {
  #[allow(missing_docs)]
  pub placeholders: Placeholders,
  #[allow(missing_docs)]
  pub replacements: Replacements<'r>,
}

#[allow(missing_docs)]
impl<'subs> FindAndReplace<'subs> {
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
    let Ok(start) = self.placeholders.automaton.start_state(Anchored::No) else {
      unreachable!("aho-corasick automaton misconfiguration");
    };
    let mut sid = start;

    for (span_idx, span) in spans.iter().enumerate() {
      // todo: quick_xml::Decoder::decode(&input[span])
      // don't decode for now, assume it's utf8
      // todo: encode replacements — should probably be done on the lower io level

      // span's space offset
      for (offset, byte) in input.as_ref()[span.clone()].iter().copied().enumerate() {
        sid = self.placeholders.automaton.next_state(Anchored::No, sid, byte);
        if !self.placeholders.automaton.is_match(sid) {
          continue;
        }

        let pat_id = self.placeholders.automaton.match_pattern(sid, 0);
        let pat_len = self.placeholders.automaton.pattern_len(pat_id);
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
          Value(xml) => xml.as_bytes(),
        };
        out.write_all(replacement)?;
        reported = span.start + r#match.end;
      }
    }

    Ok(reported)
  }
}
