# docx-template [![](https://img.shields.io/crates/v/docx-template.svg)](https://crates.io/crates/docx-template) [![](https://docs.rs/docx-template/badge.svg)](https://docs.rs/docx-template/)

Replace {placeholders} and manage content inside .docx files.

---

<img src="https://raw.githubusercontent.com/xamgore/docx-template/master/.github/assets/logo.svg" width="420" alt="Dear {crustacean}, → Dear 🦀,">

---

- **Primitive**: not a template engine, but can do quite a few transformations
  <br><sup>[Replace text](#), [swap images](#), [~~delete comments~~](#), [~~flip
  checkboxes~~](https://github.com/xamgore/docx-template/issues/6), [insert custom markup](https://github.com/xamgore/docx-template/issues/3)</sup>

- **Fast**: single-pass, avoids recompression, uses Aho-Corasick internally, almost O(n)
  <br><sup>No [long time read issues](https://github.com/bokuweb/docx-rs/issues/757) like docx-rs has</sup>

- **Not memory-efficient (yet)**: operates on a byte stream without DOM tree allocation
  <br><sup>Keeps the whole text in-memory</sup>

### Example

```rust
#[derive(serde::Serialize)]
struct Data { crustacean: String }

let data = Data { crustacean: "🦀".into() };

let output = DocxFile::from_path("in.docx")?.into_template()?.render()?;

std::fs::write("msg.docx", output)?;
```

### Features

- `serde` (default) — use `json!` macro & `Serialize` structs to create templates
- `docx-rust` — use DOM from `@cstkingkey/docx‑rust` to insert markup
- `docx-rs` — use DOM from `@bokuweb/docx‑rs` to insert markup

### Ecosystem

|                                                 name                                                  | description                                                                                                                                                                                                                                                     |
|:-----------------------------------------------------------------------------------------------------:|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|    @bokuweb/[docx&#x2011;rs](https://lib.rs/crates/docx-rs)<br>`0.4.17`<br><sup>Apr 26, 2024</sup>    | <ul><li>DOM tree (nice naming, limited navigation)</li><li>Great for write-only scenarios</li><li>🔥 WebAssembly support</li><li>A lot of examples</li><li>Compiles too many image codecs</li><li>Internal references and indexes is your own concern</li></ul> |
| @cstkingkey/[docx&#x2011;rust](https://lib.rs/crates/docx-rust)<br>`0.1.8`<br><sup>May 21, 2024</sup> | <ul><li>DOM tree (close to spec naming, conversion helpers)</li><li>Minimal dependencies</li><li>Internal references and indexes is your own concern</li></ul>                                                                                                  |
|          @yūdachi/[docx](https://lib.rs/crates/docx)<br>`1.1.2`<br><sup>(Apr 27, 2020)</sup>          | 💀 (replaced by docx-rust)                                                                                                                                                                                                                                      |
|                                                                                                       |                                                                                                                                                                                                                                                                 |
|      @kaisery/[ooxmlsdk](https://lib.rs/crates/ooxmlsdk)<br>`0.1.16`<br><sup>Oct 12, 2024</sup>       | <ul><li>Inspired by .NET [Open XML SDK](https://github.com/dotnet/Open-XML-SDK)</li><li>Low-level, generated from specification</li><li>Early development stage</ul>                                                                                            |
|               [ooxml](https://lib.rs/crates/ooxml)<br>`0.2.8`<br><sup>Nov 7, 2023</sup>               | <ul><li>Low-level</li><li>Only .xslx parsing</li></ul>                                                                                                                                                                                                          |
|                                                                                                       |                                                                                                                                                                                                                                                                 |
|                         [office-crypto](https://lib.rs/crates/office-crypto)                          | Allows decrypting password protected MS Office files                                                                                                                                                                                                            |
|                   [ms-offcrypto-writer](https://lib.rs/crates/ms-offcrypto-writer)                    | Encrypting ECMA376/OOXML files with agile encryption                                                                                                                                                                                                            |

> [!NOTE]
> Office Open XML (also informally known as OOXML or Microsoft Open XML (MOX)) is a zipped, XML-based file format developed by Microsoft for representing spreadsheets, charts, presentations and word processing documents. The format was initially standardized by Ecma (as ECMA-376), and by the ISO and IEC (as ISO/IEC 29500) in later versions.
