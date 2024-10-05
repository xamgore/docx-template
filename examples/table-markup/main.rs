use std::fs::File;

use docx_template::DocxFile;

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

  DocxFile::from_path("./examples/table-markup/input.docx")?
    .into_template_having_brackets("{", "}", &fields)?
    .render_to(File::create("./examples/table-markup/output.docx")?)?;

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
