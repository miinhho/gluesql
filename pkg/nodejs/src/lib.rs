#![deny(clippy::all)]

mod payload;
mod utils;

use {
  gluesql_core::prelude::{execute, parse, plan, translate},
  gluesql_memory_storage::MemoryStorage,
  napi::{Env, Task},
  napi_derive::napi,
  payload::{convert, GlueSQLPayload},
  std::{cell::RefCell, rc::Rc},
};

#[napi(object)]
pub struct Glue {
  storage: Rc<RefCell<Option<MemoryStorage>>>,
}

impl Default for Glue {
  fn default() -> Self {
    Self::new()
  }
}

#[napi]
impl Glue {
  #[napi(constructor)]
  pub fn new() -> Self {
    utils::set_panic_hook();

    let storage = MemoryStorage::default();

    let storage = Rc::new(RefCell::new(Some(storage)));

    Self { storage }
  }

  // TODO(@miinhho) : handle async task with tokio
  #[napi]
  pub async fn query(&mut self, env: Env, sql: String) -> napi::Result<GlueSQLPayload> {
    let cell = Rc::clone(&self.storage);

    let queries = parse(&sql).map_err(|error| format!("{error}"))?;

    let mut payloads = vec![];
    let mut storage = cell.replace(None).unwrap();

    for query in queries.iter() {
      let statement = translate(query);
      let statement = match statement {
        Ok(statement) => statement,
        Err(error) => {
          cell.replace(Some(storage));

          return Err(format!("{error}"));
        }
      };
      let statement = plan(&storage, statement).await;
      let statement = match statement {
        Ok(statement) => statement,
        Err(error) => {
          cell.replace(Some(storage));

          return Err(format!("{error}"));
        }
      };

      let result = execute(&mut storage, &statement)
        .await
        .map_err(|error| format!("{error}"));

      match result {
        Ok(payload) => {
          payloads.push(payload);
        }
        Err(error) => {
          cell.replace(Some(storage));

          return Err(error);
        }
      };
    }

    cell.replace(Some(storage));

    Ok(convert(env, payloads))
  }
}
