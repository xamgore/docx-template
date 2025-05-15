use std::io::Read;
use zip::read::ZipFile;
use zip::write::{FileOptions, SimpleFileOptions};
use zip::DateTime;

pub trait ZipFileExt
where
  Self: Sized,
{
  fn to_options(&self) -> FileOptions<()>;
}

impl<'a, R: Read> ZipFileExt for ZipFile<'a, R> {
  /// `zip` package does not provide a way to copy a file header from another archive,
  /// that's why we do it manually. Implementation is based on
  /// [ZipWriter::raw_copy_file_rename](zip::ZipWriter::raw_copy_file_rename) method.
  fn to_options(&self) -> FileOptions<()> {
    const ZIP64_BYTES_THR: u64 = u32::MAX as u64;
    const S_IFREG: u32 = 0o0100000;

    SimpleFileOptions::default()
      .large_file(self.compressed_size().max(self.size()) > ZIP64_BYTES_THR)
      .last_modified_time(
        self.last_modified().filter(DateTime::is_valid).unwrap_or_else(DateTime::default_for_write),
      )
      .compression_method(self.compression())
      .unix_permissions(self.unix_mode().unwrap_or(0o644) | S_IFREG)
  }
}
