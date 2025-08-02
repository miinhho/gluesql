use {
  gluesql_core::prelude::{Payload, PayloadVariable},
  napi::Env,
  napi_derive::napi,
  serde_json::{json, Value as Json},
};

#[napi(object)]
pub struct GlueSQLPayload {
  pub types: String,
  pub affected: Option<u32>,
  pub rows: Option<Json>,
  pub version: Option<String>,
  pub tables: Option<Vec<String>>,
  pub functions: Option<Vec<String>>,
}

pub fn convert(env: Env, payloads: Vec<Payload>) -> GlueSQLPayload {
  let payloads = payloads.into_iter().map(convert_payload).collect();
  let payloads = Json::Array(payloads);

  env.to_js_value(&payloads)
}

fn convert_payload(payload: Payload) -> Json {
  match payload {
    Payload::Create => json!({ "types": "CREATE TABLE" }),
    Payload::DropTable(num) => json!({ "types": "DROP TABLE", "affected": num }),
    Payload::Select { labels, rows } => {
      let rows = rows
        .into_iter()
        .map(|values| {
          let row = labels
            .iter()
            .zip(values.into_iter())
            .map(|(label, value)| {
              let key = label.to_owned();
              let value = Json::try_from(value).unwrap();

              (key, value)
            })
            .collect();

          Json::Object(row)
        })
        .collect();

      json!({
          "types": "SELECT",
          "rows": Json::Array(rows),
      })
    }
    Payload::SelectMap(rows) => {
      let rows = rows
        .into_iter()
        .map(|row| {
          let row = row
            .into_iter()
            .map(|(key, value)| {
              let value = Json::try_from(value).unwrap();

              (key, value)
            })
            .collect();

          Json::Object(row)
        })
        .collect();

      json!({
          "types": "SELECT",
          "rows": Json::Array(rows),
      })
    }
    Payload::ShowColumns(columns) => {
      let columns = columns
        .into_iter()
        .map(|(name, data_type)| {
          json!({
              "name": name,
              "types": data_type.to_string(),
          })
        })
        .collect();

      json!({
          "types": "SHOW COLUMNS",
          "columns": Json::Array(columns),
      })
    }
    Payload::Insert(num) => json!({
        "types": "INSERT",
        "affected": num
    }),
    Payload::Update(num) => json!({
        "types": "UPDATE",
        "affected": num
    }),
    Payload::Delete(num) => json!({
        "types": "DELETE",
        "affected": num
    }),
    Payload::AlterTable => json!({ "types": "ALTER TABLE" }),
    Payload::CreateIndex => json!({ "types": "CREATE INDEX" }),
    Payload::DropIndex => json!({ "types": "DROP INDEX" }),
    Payload::StartTransaction => json!({ "types": "BEGIN" }),
    Payload::Commit => json!({ "types": "COMMIT" }),
    Payload::Rollback => json!({ "types": "ROLLBACK" }),
    Payload::ShowVariable(PayloadVariable::Version(version)) => {
      json!({
          "types": "SHOW VERSION",
          "version": version
      })
    }
    Payload::ShowVariable(PayloadVariable::Tables(table_names)) => {
      json!({
          "types": "SHOW TABLES",
          "tables": table_names
      })
    }
    Payload::DropFunction => json!({ "types": "DROP FUNCTION" }),
    Payload::ShowVariable(PayloadVariable::Functions(function_names)) => {
      json!({
          "types": "SHOW FUNCTIONS",
          "functions": function_names
      })
    }
  }
}
