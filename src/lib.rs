#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
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

#[doc(inline)]
pub use docx_file::DocxFile;
#[doc(inline)]
pub use docx_template::{CantRenderError, DocxTemplate};
#[doc(inline)]
#[cfg(feature = "docx-rust")]
pub use markup_node::MarkupNode;
use thiserror::Error;
#[doc(hidden)]
pub use transformers::find_and_replace::FindAndReplace;
#[doc(inline)]
pub use transformers::find_and_replace::{Placeholders, Replacements, Value};

#[allow(missing_docs)]
#[derive(Error, Debug)]
#[error(transparent)]
pub struct CantSerializeError(#[from] serde_json::Error);
