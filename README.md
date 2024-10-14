# docx-template [![](https://img.shields.io/crates/v/docx-template.svg)](https://crates.io/crates/docx-template) [![](https://docs.rs/docx-template/badge.svg)](https://docs.rs/docx-template/)

Replace {placeholders} and manage content inside .docx files.

---

<img src="https://raw.githubusercontent.com/xamgore/docx-template/master/.github/assets/logo.svg" width="420" alt="Dear {crustacean}, → Dear 🦀,">

---

- **Primitive**: not a template engine, but can do quite a few transformations
  <br><sup>[Replace text](#), [swap images](#), [~~delete comments~~](#), [~~flip
  checkboxes~~](https://github.com/xamgore/docx-template/issues/6), [insert custom markup](https://github.com/xamgore/docx-template/issues/3)</sup>

- **Fast**: single-pass, avoids recompression, uses Aho-Corasick internally, almost O(n)
  <br><sup>No [long time read issues](https://github.com/bokuweb/docx-rs/issues/757) like docx&#x2011;rs has</sup>

- **Not memory-efficient (yet)**: operates on a byte stream without DOM tree allocation
  <br><sup>Keeps the whole text in&#x2011;memory</sup>

### Example

```rust
use docx_template::DocxFile;
use serde::Serialize;
use std::error::Error;

fn main() -> Result<(), Box<Error>> {
    let data = Data { crustacean: "🦀".into() };
    let output = DocxFile::from_path("in.docx")?.into_template(data)?.render()?;
    std::fs::write("output.docx", output)?;
    Ok(())
}

#[derive(Serialize)]
struct Data {
    crustacean: String
}
```

### Why

A naive approach to the problem is just calling `xml.replace("{placeholder}", "🦀")`.
Which isn't 100% accurate, as placeholders can reside in multiple adjacent XML nodes like in the example below.
That's why this crate was made. It reads XML nodes, detects patterns, and applies transformations keeping the structural integrity.

```xml
<w:run>{place</w:run><w:run>holder}</w:run>
<w:run>🦀</w:run><w:run></w:run>
```

### Features

- `serde` (default) — use `json!` macro & `Serialize` structs to create templates
- `docx-rs` — insert markup defined by @bokuweb/[docx&#x2011;rs](https://lib.rs/crates/docx-rs)
- `docx-rust` — insert markup defined by @cstkingkey/[docx&#x2011;rust](https://lib.rs/crates/docx-rust)

### Ecosystem

|                                                 name                                                  | description                                                                                                                                                                                                                                                                                                                                      |
|:-----------------------------------------------------------------------------------------------------:|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|    @bokuweb/[docx&#x2011;rs](https://lib.rs/crates/docx-rs)<br>`0.4.18`<br><sup>Apr 26, 2024</sup>    | <ul><li>DOM tree contains _owned_ values</li><li>User-friendly naming and structure</li><li>Reference ids must be set manually, despite automatic increments</li><li>🔥 Lots of <a href="https://github.com/bokuweb/docx-rs/tree/main/docx-core/examples">examples</a></li></ul><pre lang="rust">docx-template = { feature = ["docx-rs"] }</pre> |
| @cstkingkey/[docx&#x2011;rust](https://lib.rs/crates/docx-rust)<br>`0.1.8`<br><sup>May 21, 2024</sup> | <ul><li>DOM tree has a _\'lifetime_ parameter</li><li>Close-to-spec naming and structure</li><li>Reference ids must be set manually</li></ul><pre lang="rust">docx-template = { feature = ["docx-rust"] }</pre>                                                                                                                                  |
|           @yūdachi/[docx](https://lib.rs/crates/docx)<br>`1.1.2`<br><sup>Apr 27, 2020</sup>           | 💀 (forked by docx-rust)                                                                                                                                                                                                                                                                                                                         |
|                                                                                                       |                                                                                                                                                                                                                                                                                                                                                  |
|      @kaisery/[ooxmlsdk](https://lib.rs/crates/ooxmlsdk)<br>`0.1.16`<br><sup>Oct 12, 2024</sup>       | <ul><li>Inspired by .NET [Open XML SDK](https://github.com/dotnet/Open-XML-SDK)</li><li>Low-level, generated from specification</li><li>Early development stage</ul>                                                                                                                                                                             |
|                                                                                                       |                                                                                                                                                                                                                                                                                                                                                  |
|                         [office-crypto](https://lib.rs/crates/office-crypto)                          | Allows decrypting password protected MS Office files                                                                                                                                                                                                                                                                                             |
|                   [ms-offcrypto-writer](https://lib.rs/crates/ms-offcrypto-writer)                    | Encrypting ECMA376/OOXML files with agile encryption                                                                                                                                                                                                                                                                                             |

> [!NOTE]
> Office Open XML (also informally known as OOXML or Microsoft Open XML (MOX)) is a zipped, XML-based file format
> developed by Microsoft for representing spreadsheets, charts, presentations and word processing documents. The format
> was initially standardized by Ecma (as ECMA-376), and by the ISO and IEC (as ISO/IEC 29500) in later versions.
