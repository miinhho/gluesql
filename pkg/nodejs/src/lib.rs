#![deny(clippy::all)]

mod payload;
mod utils;

use {
  gluesql_core::prelude::{execute, parse, plan, translate},
  gluesql_memory_storage::MemoryStorage,
  napi::{Env, bindgen_prelude::Null},
  napi_derive::napi,
  payload::{convert, GlueSQLPayload},
  std::{cell::RefCell, rc::Rc},
};

#[cfg(target_family = "wasm")]
use {
    gluesql_composite_storage::CompositeStorage,
    gluesql_idb_storage::IdbStorage,
    gluesql_web_storage::{WebStorage, WebStorageType},
};

#[napi]
pub struct Glue {
  #[cfg(target_family = "wasm")]
  storage: Rc<RefCell<Option<CompositeStorage>>>,

  #[cfg(not(target_family = "wasm"))]
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

    #[cfg(target_family = "wasm")]
    let storage = {
        let mut storage = CompositeStorage::default();
        storage.push("memory", MemoryStorage::default());
        storage.push("localStorage", WebStorage::new(WebStorageType::Local));
        storage.push("sessionStorage", WebStorage::new(WebStorageType::Session));
        storage.set_default("memory");
        // debug("[GlueSQL] loaded: memory, localStorage, sessionStorage");
        // debug("[GlueSQL] default engine: memory");

        storage
    };

    #[cfg(not(target_family = "wasm"))]
    let storage = MemoryStorage::default();

    let storage = Rc::new(RefCell::new(Some(storage)));

    Self { storage }
  }

  #[cfg(target_family = "wasm")]
  #[napi(js_name = loadIndexedDB)]
  pub async unsafe fn load_indexeddb(&mut self, env: Env, namespace: Option<String>) -> Promise {
      let cell = Rc::clone(&self.storage);

      tokio::task::spawn(async move {
          let mut storage = cell.replace(None).unwrap();

          if storage.storages.contains_key("indexedDB") {
              cell.replace(Some(storage));

              return Err(env.create_string("indexedDB storage is already loaded"));
          }

          let idb_storage = match IdbStorage::new(namespace).await {
              Ok(storage) => storage,
              Err(error) => {
                  cell.replace(Some(storage));

                  return Err(env.create_string(format!("{error}")));
              }
          };

          storage.push("indexedDB", idb_storage);
          // debug("[GlueSQL] loaded: indexedDB");

          cell.replace(Some(storage));

          Ok(Null)
      })
  }

  #[cfg(target_family = "wasm")]
  #[napi(js_name = setDefaultEngine)]
  pub fn set_default_engine(&mut self, env: Env, default_engine: String) -> Result<(), JsValue> {
      let cell = Rc::clone(&self.storage);
      let mut storage = cell.replace(None).unwrap();

      let result = {
          if !["memory", "localStorage", "sessionStorage", "indexedDB"]
              .iter()
              .any(|engine| engine == &default_engine.as_str())
          {
              Err(env.create_string(
                  format!("{default_engine} is not supported (options: memory, localStorage, sessionStorage, indexedDB)").as_str()
              ))
          } else if default_engine == "indexedDB" && !storage.storages.contains_key("indexedDB") {
              Err(env.create_string(
                  "indexedDB is not loaded - run loadIndexedDB() first",
              ))
          } else {
              storage.set_default(default_engine);

              Ok(())
          }
      };

      cell.replace(Some(storage));
      result
  }

  #[napi]
  pub async unsafe fn query(&mut self, env: Env, sql: String) -> napi::Result<GlueSQLPayload> {
    let cell = Rc::clone(&self.storage);

    tokio::task::spawn(async move {
      let queries = parse(&sql).map_err(|error| format!("{error}"))?;

      let mut payloads = vec![];
      let mut storage = cell.replace(None).unwrap();

      for query in queries.iter() {
        let statement = translate(query);
        let statement = match statement {
          Ok(statement) => statement,
          Err(error) => {
            cell.replace(Some(storage));

            return Err(env.create_string(format!("{error}")));
          }
        };
        let statement = plan(&storage, statement).await;
        let statement = match statement {
          Ok(statement) => statement,
          Err(error) => {
            cell.replace(Some(storage));

            return Err(env.create_string(format!("{error}")));
          }
        };

        let result = execute(&mut storage, &statement)
          .await
          .map_err(|error| env.create_string(format!("{error}")));

        match result {
          Ok(payload) => {
            payloads.push(payload);
          }
          Err(error) => {
            cell.replace(Some(storage));

            return Err(env.create_string(format!("{error}")));
          }
        };
      }

      cell.replace(Some(storage));

      Ok(convert(env, payloads))
    })
  }
}
