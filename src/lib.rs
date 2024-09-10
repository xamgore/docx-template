#[macro_use]
mod regex;

#[cfg(feature = "docx-rust")]
mod doc_layout_node;
pub mod docx_file;
mod docx_part;
pub mod docx_template;
mod fmt_to_io_adapter;
pub mod transformers;
mod zip_file_ext;

pub use docx_file::DocxFile;
pub use docx_template::{DocxTemplate, DocxTemplateError};
