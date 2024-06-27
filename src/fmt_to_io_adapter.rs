use std::fmt::Formatter;
use std::io;
use std::str::from_utf8;

pub struct IntoIoAdapter<'a, 'b> {
  formatter: &'a mut Formatter<'b>,
}

impl<'a, 'b> From<&'a mut Formatter<'b>> for IntoIoAdapter<'a, 'b> {
  fn from(formatter: &'a mut Formatter<'b>) -> Self {
    IntoIoAdapter { formatter }
  }
}

impl io::Write for IntoIoAdapter<'_, '_> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let str = from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    self.formatter.write_str(str).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(buf.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
