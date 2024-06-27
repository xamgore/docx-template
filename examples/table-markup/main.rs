use std::fs::File;
use std::io::BufWriter;

use docx_template::docx_file::DocxFile;
use docx_template::docx_template::DocxTemplate;
use docx_template::transformers::find_and_replace::{Patterns, Replacement};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let fields = Fields {
    dos_num: 1423,
    dt: chrono::Local::now().naive_local().date(),
    clientName: Some(r#"ООО “Приоритетная логистика”"#.into()),
    clientContact: Some("Иван Иванович Иванов".into()),
    clientPhone: Some("+7 (499) 455-01-68".into()),
    clientMail: Some("logistics@priorlog.com".into()),
    supplierName: Some(r#"ПАО "АЭРОФЛОТ""#.into()),
    load_dt: chrono::Local::now().naive_utc(),
    unload_dt: chrono::Local::now().naive_local(),
    consigneeName: r#"ООО "ИНТЕЛЛЕКТУАЛЬНЫЕ ТЕХНОЛОГИИ""#.into(),
    consigneeAddress: r#"117437, город Москва, ул. Академика Арцимовича, дом 17, Э 1 П V К 9 ОФ 130"#.into(),
    senderName: (r#"ПАО "РОСТЕЛЕКОМ""#.into()),
    supplierPhone: Some("+7 (499) 455-01-68".into()),
    senderAddress: (r#"191167, город Санкт-Петербург, вн.тер. г. Муниципальный Округ Смольнинское, наб Синопская, дом 14, литера А"#.into()),
    customsPoint: "✕".into(),
    customsImport: "✕".into(),
    cargo: "СОРМ роутер".into(),
    weight: "20 кг".into(),
    volume: "0.4 м³".into(),
    truckType: "Высокая фура (3м, 100м3)".into(),
    parcels: 1,
    insurance: "Не требуется".into(),
    goodsPrice:  "1'000'000 RUB".into(),
    truckNumber: "с065мк 78".into(),
    driverName: "Вахтанг Альбертович".into(),
    ..Default::default()
  };

  let serde_json::Value::Object(map) = serde_json::to_value(fields)? else { panic!() };

  let path = "./examples/table-markup/input.docx";
  let mut template = DocxTemplate {
    template: DocxFile::from_path(path)?,
    patterns: Patterns::from_iter_with_brackets("{", "}", map.keys()),
  };

  let path = "./examples/table-markup/output.docx";
  let mut result = BufWriter::new(File::create(path).unwrap());
  let replacements: Vec<_> = map.values().cloned().map(Replacement::from).collect();
  template.render(&mut result, &replacements)?;

  Ok(())
}

#[allow(non_snake_case)]
#[derive(Default, serde::Serialize)]
struct Fields {
  dos_num: u64,
  dt: chrono::NaiveDate,
  contract: Option<String>,
  clientName: Option<String>,
  clientContact: Option<String>,
  clientPhone: Option<String>,
  clientMail: Option<String>,
  supplierName: Option<String>,
  supplierContact: Option<String>,
  supplierPhone: Option<String>,
  supplierMail: Option<String>,
  load_dt: chrono::NaiveDateTime,
  unload_dt: chrono::NaiveDateTime,
  senderName: String,
  senderAddress: String,
  senderContacts: String,
  customsPoint: String,
  customsImport: String,
  consigneeName: String,
  consigneeAddress: String,
  consigneeContacts: String,
  cargo: String,
  weight: String,
  volume: String,
  truckType: String,
  parcels: i64,
  insurance: String,
  goodsPrice: String,
  extraInfo: Option<String>,
  amount: String,
  extraAmount: String,
  truckNumber: String,
  driverName: String,
  clientDispatcher: String,
  supplierDispatcher: String,
}
