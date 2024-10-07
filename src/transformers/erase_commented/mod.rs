#![allow(dead_code)]

use std::collections::HashSet;
use std::io;
use std::ops::Range;

use quick_xml::events::Event;

pub struct EraseCommentedRangeTransformerBrw<'i> {
  xml: quick_xml::reader::Reader<&'i [u8]>,
  #[allow(dead_code)]
  comment_ids: HashSet<Vec<u8>>,
  event: Event<'i>,
  cur: Range<usize>,
}

impl<'i> EraseCommentedRangeTransformerBrw<'i> {
  pub fn from_reader(reader: &'i [u8]) -> Self {
    Self {
      xml: quick_xml::Reader::from_reader(reader),
      comment_ids: Default::default(),
      event: Event::Eof,
      cur: 0..0,
    }
  }
}

mod joined_str {
  pub struct JoinedStr<'a> {
    prefix: &'static str,
    infix: &'a str,
    suffix: &'static str,
  }

  impl<'a> JoinedStr<'a> {
    pub fn new(prefix: &'static str, infix: &'a str, suffix: &'static str) -> Self {
      Self { prefix, infix, suffix }
    }
  }
}

fn prefix(e: &Event) -> &'static [u8] {
  match e {
    Event::Start(_) => b"<",
    Event::End(_) => b"</",
    Event::Empty(_) => b"<",
    Event::Text(_) => b"",
    Event::CData(_) => b"<![CDATA[",
    Event::Comment(_) => b"<!--",
    Event::Decl(_) => b"<?",
    Event::PI(_) => b"<?",
    Event::DocType(_) => b"<!DOCTYPE ",
    Event::Eof => b"",
  }
}

fn suffix(e: &Event) -> &'static [u8] {
  match e {
    Event::Start(_) => b">",
    Event::End(_) => b">",
    Event::Empty(_) => b"/>",
    Event::Text(_) => b"",
    Event::CData(_) => b"]]>",
    Event::Comment(_) => b"-->",
    Event::Decl(_) => b"?>",
    Event::PI(_) => b"?>",
    Event::DocType(_) => b">",
    Event::Eof => b"",
  }
}

impl<'i> io::Read for EraseCommentedRangeTransformerBrw<'i> {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let mut cur = self.cur.clone();

    if cur.is_empty() {
      self.event = self.xml.read_event().map_err(io::Error::other)?;
      cur = 0..(prefix(&self.event).len() + self.event.len() + suffix(&self.event).len());
    }

    let prefix = prefix(&self.event);
    let suffix = suffix(&self.event);

    let count = if cur.start < prefix.len() {
      let head_size = prefix.len().min(buf.len());
      let head = cur.start..(cur.start + head_size);
      let tail = (cur.start + head_size)..cur.end;
      buf[..head_size].copy_from_slice(&prefix[head]);
      self.cur = tail;
      head_size
    } else if cur.start < prefix.len() + self.event.len() {
      let head_size = self.event.len().min(buf.len());
      let head = (cur.start - prefix.len())..(cur.start - prefix.len() + head_size);
      let tail = (cur.start + head_size)..cur.end;
      buf[..head_size].copy_from_slice(&self.event[head]);
      self.cur = tail;
      head_size
    } else {
      let head_size = suffix.len().min(buf.len());
      let head = (cur.start - prefix.len() - self.event.len())
        ..(cur.start - prefix.len() - self.event.len() + head_size);
      let tail = (cur.start + head_size)..cur.end;
      buf[..head_size].copy_from_slice(&suffix[head]);
      self.cur = tail;
      head_size
    };

    Ok(count)
  }
}

pub struct EraseCommentedRangeTransformer<R> {
  input: R,
  xml: quick_xml::reader::Reader<R>,
  #[allow(dead_code)]
  comment_ids: HashSet<Vec<u8>>,
  offset: usize,
  cur: Range<usize>,
}

impl<R: Clone> EraseCommentedRangeTransformer<R> {
  pub fn from_reader(reader: R) -> Self {
    Self {
      input: reader.clone(),
      xml: quick_xml::Reader::from_reader(reader),
      comment_ids: Default::default(),
      offset: 0,
      cur: 0..0,
    }
  }

  // fn has_attribute_in_block_list(&self, attrs: Attributes) -> bool {
  //   attrs
  //     .flatten()
  //     .any(|attr| attr.key.as_ref() == b"w:id" && self.comment_ids.contains(attr.value.as_ref()))
  // }
}

impl<'i> io::Read for EraseCommentedRangeTransformer<&'i [u8]> {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let mut cur = self.cur.clone();

    if cur.is_empty() {
      let _event = self.xml.read_event().map_err(io::Error::other)?;
      cur = self.offset..self.xml.buffer_position() as usize;
      self.offset = cur.end;
    }

    let head_size = cur.len().min(buf.len());
    let head = cur.start..(cur.start + head_size);
    let tail = head.end..cur.end;

    let text = &self.input[head];
    buf[..head_size].copy_from_slice(text);
    self.cur = tail;

    Ok(head_size)

    // let mut skip = false;
    //
    // loop {
    //   event.
    //
    //   match event {
    //     Event::Start(tag) if tag.name().as_ref() == b"w:commentRangeStart" && self.has_attribute_in_block_list(tag.attributes()) => {
    //       skip = true
    //     },
    //     Event::End(tag) if tag.name().as_ref() == b"w:commentRangeEnd" => {}
    //
    //
    //     // Event::Start(tag) => match tag.name().as_ref() {
    //     //   b"w:commentRangeStart" if self.has_attribute_in_block_list(tag.attributes()) => {
    //     //     skip = true;
    //     //   }
    //     //   _ => {}
    //     // },
    //     Event::Eof => return Ok(0),
    //     _ => {
    //       todo!()
    //     }
    //   }
    // }
  }
}

#[cfg(test)]
mod tests {
  use std::io::Read;

  use crate::transformers::erase_commented::{
    EraseCommentedRangeTransformer, EraseCommentedRangeTransformerBrw,
  };

  #[test]
  fn trans1() -> Result<(), Box<dyn std::error::Error>> {
    let template =
      include_bytes!("../../../features/comment_and_image/word/document.xml").as_slice();
    let mut transformer = EraseCommentedRangeTransformer::from_reader(template);
    let mut buf = String::new();
    transformer.read_to_string(&mut buf)?;
    println!("{}", buf.replace('\r', "\n"));
    // std::io::copy(&mut transformer, &mut std::io::stdout())?;
    Ok(())
  }

  #[test]
  #[ignore]
  fn trans2() -> Result<(), Box<dyn std::error::Error>> {
    let template =
      include_bytes!("../../../features/comment_and_image/word/document.xml").as_slice();
    let mut transformer = EraseCommentedRangeTransformerBrw::from_reader(template);
    let mut buf = String::new();
    transformer.read_to_string(&mut buf)?;
    println!("{}", buf.replace('\r', "\n"));
    // std::io::copy(&mut transformer, &mut std::io::stdout())?;
    Ok(())
  }
}
