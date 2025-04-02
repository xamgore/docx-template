use self::helpers::*;
use docx_rs::{DocumentChild, Paragraph, Run, Table, TableCell, TableLayoutType, TableRow};
use docx_template::{DocxFile, DocxRsMarkupNode, DocxTemplate, Placeholders, Replacements};
use std::fs::File;
use std::io::BufWriter;
use std::iter::{once, repeat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let path = "./examples/insert-table/input.docx";
  let file = DocxFile::from_path(path)?;

  let path = "./examples/insert-table/output.docx";
  let result = BufWriter::new(File::create(path).unwrap());

  DocxTemplate::new(
    file,
    Placeholders::from_iter(["{table}"]),
    Replacements::from_iter([DocxRsMarkupNode::InBody(vec![DocumentChild::Table(Box::new(
      table(),
    ))])]),
  )
  .render_to(result)?;

  Ok(())
}

pub fn table() -> Table {
  let data = [
    Item::new("Product A", 5, 10.),
    Item::new("Product B", 2, 15.),
    Item::new("Product C", 3, 8.5),
    Item::new("Product D", 1, 20.),
    Item::new("Product E", 4, 12.75),
    Item::new("Product F", 2, 18.5),
    Item::new("Product G", 6, 9.25),
    Item::new("Product H", 3, 14.),
    Item::new("Product I", 2, 17.),
    Item::new("Product J", 1, 25.),
  ];

  Table::new(
    // header + content + summary footer
    once(Item::header())
      .chain(data.iter().enumerate().map(|(idx, it)| it.to_row(1 + idx)))
      .chain(once(Item::summary(&data)))
      .collect(),
  )
  .set_grid(repeat(1960).take(5).collect())
  .layout(TableLayoutType::Autofit)
}

struct Item {
  description: String,
  quantity: i64,
  unit_price: f64,
}

impl Item {
  fn new(desc: &str, qty: i64, price: f64) -> Self {
    Self { description: desc.to_owned(), quantity: qty, unit_price: price }
  }

  fn total(&self) -> f64 {
    self.quantity as f64 * self.unit_price
  }

  fn header() -> TableRow {
    TableRow::new(
      ["Item", "Description", "Quantity", "Unit Price ($)", "Total ($)"].map(head_cell).to_vec(),
    )
  }

  fn to_row(&self, idx: usize) -> TableRow {
    TableRow::new(vec![
      cell(idx),
      cell(&self.description),
      cell(self.quantity),
      cell(self.unit_price),
      cell(self.total()),
    ])
  }

  fn summary<'a>(items: impl IntoIterator<Item = &'a Self>) -> TableRow {
    let total: f64 = items.into_iter().map(|it| it.total()).sum();
    TableRow::new(vec![empty(), bold("Total"), empty(), empty(), bold(total)])
  }
}

mod helpers {
  use super::*;
  use docx_rs::{AlignmentType, Shading, VAlignType};

  pub fn empty() -> TableCell {
    TableCell::new()
  }

  pub fn cell(label: impl ToString) -> TableCell {
    TableCell::new()
      .add_paragraph(
        Paragraph::new()
          .id(next_id())
          .align(AlignmentType::Center)
          .add_run(Run::new().add_text(label.to_string())),
      )
      .vertical_align(VAlignType::Center)
  }

  pub fn bold(label: impl ToString) -> TableCell {
    TableCell::new()
      .add_paragraph(
        Paragraph::new()
          .id(next_id())
          .align(AlignmentType::Center)
          .add_run(Run::new().add_text(label.to_string()).bold()),
      )
      .vertical_align(VAlignType::Center)
  }

  pub fn head_cell(label: impl ToString) -> TableCell {
    bold(label).shading(Shading::new().fill("D4D4D4"))
  }

  fn next_id() -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    // generated ids should not intersect with those already in the document
    static ID: AtomicUsize = AtomicUsize::new(500_000);
    ID.fetch_add(1, Ordering::Relaxed).to_string()
  }
}
