use databend_client::{APIClient, Pages};
use databend_client::{Page, QueryStats};
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::console;
use web_sys::{js_sys, MessageEvent, Worker, WorkerOptions, WorkerType};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub struct Connection {
    #[wasm_bindgen(skip)]
    client: Option<Arc<APIClient>>,
}

impl Connection {
    pub fn get_client(&self) -> Result<Arc<APIClient>, String> {
        if let Some(client) = &self.client {
            Ok(client.clone())
        } else {
            Err("not connected yet".to_string())
        }
    }
}

#[wasm_bindgen]
impl Connection {
    #[allow(unused)]
    pub fn new() -> Self {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        Self { client: None }
    }

    #[allow(unused)]
    pub async fn connect(&mut self, dsn: &str) -> Result<(), String> {
        self.client = Some(APIClient::new(dsn, None).await.map_err(|e| e.to_string())?);
        Ok(())
    }

    #[allow(unused)]
    pub async fn execute(
        &mut self,
        sql: &str,
        callback: js_sys::Function,
    ) -> Result<Statement, String> {
        let client = self.get_client()?;
        let pages = client
            .start_query(sql, true)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Statement::new(pages, callback).await)
    }

    #[allow(unused)]
    pub fn database(&self) -> Result<String, String> {
        let client = self.get_client()?;
        Ok(client.current_database().unwrap_or("default".to_owned()))
    }
}

#[allow(unused)]
#[wasm_bindgen]
pub struct Statement {
    #[wasm_bindgen(skip)]
    pub(crate) pages: Rc<RefCell<Pages>>,
    #[wasm_bindgen(skip)]
    callback: js_sys::Function,
    #[wasm_bindgen(skip)]
    worker: Worker,
    #[wasm_bindgen(skip)]
    on_message: Closure<dyn FnMut(MessageEvent)>,
    #[wasm_bindgen(skip)]
    stats: QueryStats,
}

impl Statement {
    #[allow(unused)]
    pub fn status(&self) -> QueryStats {
        self.stats.clone()
    }

    pub(crate) async fn new(mut pages: Pages, callback: js_sys::Function) -> Self {
        let this = JsValue::null();
        let page: Page = pages.first_page().expect("first_page should not be None");
        let stats = page.stats.clone();
        let _ = callback.call1(&this, &serde_json::to_string(&page).unwrap().into());
        let pages = Rc::new(RefCell::new(pages));

        let worker = new_worker().await;
        let pages_clone = pages.clone();
        let worker_clone = worker.clone();
        let callback_clone = callback.clone();

        // console::log_1(&"Creating closure for worker message handler".into());
        let on_message = Closure::new(move |msg: web_sys::MessageEvent| {
            let data = msg.data();

            let this = JsValue::null();
            // console::log_2(&"get message from worker".into(), &data);
            let obj = match js_sys::Object::try_from(&data) {
                Some(obj) => obj,
                None => {
                    console::error_1(&"bendsql: Failed to convert worker message to object".into());
                    return;
                }
            };

            let type_val = match js_sys::Reflect::get(&obj, &JsValue::from_str("type")) {
                Ok(val) => val,
                Err(e) => {
                    console::error_1(
                        &format!("bendsql: Failed to get worker message type: {:?}", e).into(),
                    );
                    return;
                }
            };

            let type_val = match type_val.as_string() {
                Some(s) => s,
                None => {
                    console::error_1(&"bendsql: Worker message type is not a string: {:?}".into());
                    return;
                }
            };

            let pages = &mut *pages_clone.borrow_mut();
            match type_val.as_str() {
                "ready" => {
                    //console::log_1(&"databend worker is ready, sending message".into());
                }
                "data" => {
                    if let Ok(body) = js_sys::Reflect::get(&obj, &JsValue::from_str("body")) {
                        if let Ok(array_buffer) = body.dyn_into::<js_sys::ArrayBuffer>() {
                            let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                            let buffer = uint8_array.to_vec();

                            let data = match pages.on_response(buffer) {
                                Ok(page) => serde_json::to_string(&page).unwrap(),
                                Err(err_msg) => {
                                    serde_json::to_string(&serde_json::json!({"error": err_msg}))
                                        .unwrap()
                                }
                            };
                            let _ = callback_clone.call1(&this, &data.into());
                        } else {
                            console::error_1(&"body is not ArrayBuffer".into());
                            return;
                        }
                    } else {
                        console::error_1(&"no field named `body`".into());
                        return;
                    }
                }
                "error" => {
                    if let Ok(error_msg) = js_sys::Reflect::get(&obj, &JsValue::from_str("error")) {
                        if let Some(error_str) = error_msg.as_string() {
                            console::error_1(&format!("Worker error: {}", error_str).into());
                        }
                    }
                }
                _ => {
                    console::error_1(&format!("Unknown message type: {}", type_val).into());
                }
            };
            if let Some((next_uri, headers)) = pages.next_uri() {
                let msg = json!({
                    "url": next_uri,
                    "headers": headers,
                })
                .to_string();
                if let Err(e) = worker_clone.post_message(&JsValue::from(msg)) {
                    console::error_1(&format!("Failed to post message to worker: {:?}", e).into());
                }
            } else {
                let end_msg = serde_json::to_string(&serde_json::json!({"end": true})).unwrap();
                let _ = callback_clone.call1(&this, &end_msg.into());
            }
        });

        // console::log_1(&"Setting worker onmessage handler".into());
        worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        Self {
            pages,
            callback,
            worker,
            on_message,
            stats,
        }
    }
}

impl Drop for Statement {
    fn drop(&mut self) {
        self.worker.terminate();
        // console::log_1(&"Statement dropped".into());
    }
}

pub async fn new_worker() -> Worker {
    let worker_options = WorkerOptions::new();
    worker_options.set_type(WorkerType::Module);

    let worker_js = include_str!("./resources/worker.js");
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/javascript");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(
        &js_sys::Array::of1(&JsValue::from_str(worker_js)),
        &options,
    )
    .unwrap();
    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

    let worker = Worker::new_with_options(&url, &worker_options).unwrap();
    // console::log_1(&"Created a new worker from within Wasm".into());

    worker
}
