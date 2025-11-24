use wasm_bindgen::prelude::wasm_bindgen;
use crate::QueryResponse;
use super::client::ClientIf;
use super::client::APIClient;

#[wasm_bindgen]
struct WasmClient {
    inner: Option<Box<dyn ClientIf>>,
}

#[wasm_bindgen]
impl WasmClient {
    pub fn new() -> Self {
        Self {
            inner: None,
        }
    }

    pub async fn init(&mut self, dsn: &str) {
       self.inner = Some(Box::new(APIClient::new(dsn, None).await.unwrap()));
    }

    pub async fn query(&mut self, sql: &str) -> usize {
         let response = self.inner.as_mut().expect("client not inited").start_query(sql).await.unwrap();
        response.data.len()
    }
}