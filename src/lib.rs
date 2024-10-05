#![warn(missing_docs)]
#![deny(unused_imports)]

#[macro_use]
mod regex;

mod docx_file;
mod docx_part;
mod docx_template;
mod fmt_to_io_adapter;
#[cfg(feature = "docx-rust")]
mod markup_node;
pub(crate) mod transformers;
mod zip_file_ext;

pub use docx_file::DocxFile;
pub use docx_template::{DocxTemplate, DocxTemplateError};
#[cfg(feature = "docx-rust")]
pub use markup_node::MarkupNode;
#[doc(hidden)]
pub use transformers::find_and_replace::FindAndReplace;
pub use transformers::find_and_replace::{Placeholders, Replacements, Value};
