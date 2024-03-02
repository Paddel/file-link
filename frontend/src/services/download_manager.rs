use std::{cell::RefCell, rc::Rc};

use js_sys::Array;
use serde_derive::{Serialize, Deserialize};
use wasm_bindgen::{JsValue, prelude::Closure, JsCast};
use web_sys::{console, IdbDatabase, IdbKeyRange};

use crate::file_tag::FileTag;

const STORE_NAME: &str = "file.link.chunks";

#[derive(Clone, Serialize, Deserialize)]
pub struct FileMeta {
    name: String,
    chunks: u32,
}

pub struct DownloadManager {
    idb: Rc<RefCell<Option<IdbDatabase>>>,
    downloaded_volume: u64,
    file_tag: Option<FileTag>,
    chunk_counter: u32,
}

impl DownloadManager {
    pub fn new() -> Self {
        let idb = Rc::new(RefCell::new(None));
        
        if let Err(_) = Self::setup_idb(idb.clone()) {
            console::log_1(&JsValue::from_str(&"Error setting up IDB"));
        }

        Self {
            idb,
            downloaded_volume: 0,
            file_tag: None,
            chunk_counter: 0,
        }
    }

    fn setup_idb(idb: Rc<RefCell<Option<IdbDatabase>>>) -> Result<(), JsValue> {
        let window = Self::get_window()?;
        let idb_factory = Self::get_idb_factory(&window)?;
        _= idb_factory.delete_database("downloads");
        let request = idb_factory.open_with_u32("downloads", 1)?;
        
        Self::set_callbacks(&request, idb.clone());
        Ok(())
    }

    fn get_window() -> Result<web_sys::Window, JsValue> {
        web_sys::window().ok_or(JsValue::from_str("No global `window` exists"))
    }

    fn get_idb_factory(window: &web_sys::Window) -> Result<web_sys::IdbFactory, JsValue> {
        window.indexed_db()?.ok_or_else(|| JsValue::from_str("IndexedDB is not supported"))
    }

    fn set_callbacks(request: &web_sys::IdbOpenDbRequest, idb: Rc<RefCell<Option<IdbDatabase>>>) {
        let on_success = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let db: IdbDatabase = event.target().unwrap().dyn_into::<web_sys::IdbRequest>().unwrap().result().unwrap().dyn_into().unwrap();
            *idb.borrow_mut() = Some(db.clone());
        }) as Box<dyn FnMut(_)>);
        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        on_success.forget();
        
        let on_upgrade_needed = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let db: web_sys::IdbDatabase = event.target().unwrap().dyn_into::<web_sys::IdbRequest>().unwrap().result().unwrap().dyn_into::<web_sys::IdbDatabase>().unwrap();
            if !db.object_store_names().contains(&STORE_NAME.to_string()) {
                db.create_object_store_with_optional_parameters(
                    STORE_NAME, 
                    &web_sys::IdbObjectStoreParameters::new().auto_increment(true)
                ).unwrap();

            }
        }) as Box<dyn FnMut(_)>);
        request.set_onupgradeneeded(Some(on_upgrade_needed.as_ref().unchecked_ref()));
        on_upgrade_needed.forget();

        let on_error = Closure::wrap(Box::new(move |e: web_sys::Event| {
            console::log_1(&JsValue::from_str(&format!("Error opening downloads database: {:?}", e)));
        }) as Box<dyn FnMut(_)>);
        
        request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();

        let on_blocked = Closure::wrap(Box::new(move |e: web_sys::Event| {
            console::log_1(&JsValue::from_str(&format!("Blocked opening downloads database: {:?}", e)));
        }) as Box<dyn FnMut(_)>);

        request.set_onblocked(Some(on_blocked.as_ref().unchecked_ref()));
        on_blocked.forget();
    }

    pub fn active(&self) -> bool {
        if let Some(file_tag) = &self.file_tag {
            self.downloaded_volume < file_tag.size as u64
        } else {
            false
        }
    }

    pub fn get_file_tag(&self) -> Option<FileTag> {
        self.file_tag.clone()
    }

    pub fn get_progress(&self) -> f64 {
        if let Some(file_tag) = &self.file_tag {
            self.downloaded_volume as f64 / file_tag.size as f64
        } else {
            0.0
        }
    }

    pub fn new_file(&mut self, file_tag: FileTag) {
        self.file_tag = Some(file_tag);
        self.downloaded_volume = 0;
        self.chunk_counter = 0;
    }

    pub fn save_chunk(&mut self, chunk: &JsValue, size: u32) -> Result<bool, JsValue> {
        if self.file_tag.is_none() {
            return Err(JsValue::from_str("No file tag set"));
        }

        let file_tag = self.file_tag.clone().unwrap();
        let transaction = self.idb
            .borrow();

        if transaction.is_none() {
            return Err(JsValue::from_str("No IDB transaction"));
        }

        let transaction = transaction
            .as_ref()
            .unwrap()
            .transaction_with_str_and_mode(STORE_NAME, web_sys::IdbTransactionMode::Readwrite)?;

        let store: web_sys::IdbObjectStore = transaction.object_store(STORE_NAME)?;
        let key = JsValue::from_str(&format!("${}-${}", self.file_tag.as_ref().unwrap().uuid(), self.chunk_counter));
        _= store.put_with_key(chunk, &key)?;
        self.downloaded_volume += size as u64;
        self.chunk_counter += 1;
        
        if self.downloaded_volume == file_tag.size as u64 {
            self.download(self.file_tag.clone().unwrap());
            let key_meta = JsValue::from_str(&format!("${}", self.file_tag.as_ref().unwrap().uuid()));
            let info_meta = serde_json::to_string(&FileMeta {
                name: file_tag.name().to_string(),
                chunks: self.chunk_counter,
            }).unwrap();
            let info_meta = JsValue::from_str(&info_meta);
            _ = store.put_with_key(&info_meta, &key_meta);
            return Ok(true);
        }
        Ok(false)
    }

    pub fn download(&self, file_tag: FileTag) {
        let transaction = self.idb
            .borrow()
            .as_ref()
            .unwrap()
            .transaction_with_str_and_mode(STORE_NAME, web_sys::IdbTransactionMode::Readonly).unwrap();
        let store: web_sys::IdbObjectStore = transaction.object_store(STORE_NAME).unwrap();

        let key_meta = JsValue::from_str(&format!("${}", file_tag.uuid()));
        let key_range = IdbKeyRange::only(&key_meta).unwrap();
        let store_clone = store.clone();
        let on_success: Closure<dyn FnMut(web_sys::Event)> = Closure::wrap(Box::new(move |event| {
            let cursor: Result<web_sys::IdbCursorWithValue, JsValue> = event.target().unwrap().dyn_into::<web_sys::IdbRequest>().unwrap().result().unwrap().dyn_into::<web_sys::IdbCursorWithValue>();
            if cursor.is_err() {
                return;
            }
            let cursor = cursor.unwrap();
            let value = cursor.value().unwrap();
            let meta: FileMeta = serde_json::from_str(&value.as_string().unwrap()).unwrap();
            Self::assemble_and_download(store_clone.clone(),file_tag.clone(), meta);
        }));

        store.open_cursor_with_range(&key_range).unwrap().set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        on_success.forget();
    }

    pub fn assemble_and_download(store: web_sys::IdbObjectStore, file_tag: FileTag, info_meta: FileMeta) {
        let js_blob_parts = Rc::new(RefCell::new(js_sys::Array::new()));
        for counter in 0..info_meta.chunks {
            let js_blob_parts = js_blob_parts.clone();
            let chunks = info_meta.chunks.clone();
            let key = JsValue::from_str(&format!("${}-${}", file_tag.uuid(), counter));
            let key_range = IdbKeyRange::only(&key).unwrap();
            
            let info_meta = info_meta.clone();
            let on_success: Closure<dyn FnMut(web_sys::Event)> = Closure::wrap(Box::new(move |event| {
                let result = Self::transfer_to_download(&js_blob_parts.borrow_mut(), event);
                if let Err(_err) = result {
                    // console::log_1(&format!("Error downloading file: {:?}", err).into());
                    return;
                }
                
                if counter == chunks - 1 {
                    let anchor = Self::inject_download(&js_blob_parts.borrow());
                    if let Ok(anchor) = anchor {
                        let _ = anchor.set_download(&info_meta.clone().name);
                        anchor.click();
                    }
                }
            }));
            
            store.open_cursor_with_range(&key_range).unwrap().set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
            on_success.forget();
        }
    }

    fn inject_download(js_blob_parts: &Array) -> Result<web_sys::HtmlAnchorElement, JsValue> {
        let combined_blob = web_sys::Blob::new_with_u8_array_sequence(&js_blob_parts)?;
        let url = web_sys::Url::create_object_url_with_blob(&combined_blob)?;
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let anchor = document.create_element("a")?;
        let anchor = anchor.dyn_into::<web_sys::HtmlAnchorElement>()?;
        let _ = anchor.set_href(&url);
        Ok(anchor)
    }

    fn transfer_to_download(js_blob_parts: &Array, event: web_sys::Event) -> Result<(), JsValue> {
        let cursor: Result<web_sys::IdbCursorWithValue, JsValue> = event.target().unwrap().dyn_into::<web_sys::IdbRequest>()?.result()?.dyn_into::<web_sys::IdbCursorWithValue>();
        
        if cursor.is_err() {
            return Err(JsValue::from_str("No cursor"));
        }

        let cursor = cursor.unwrap();
        let value = cursor.value()?;
        let u8_array = js_sys::Uint8Array::new(&value.dyn_into::<js_sys::ArrayBuffer>().unwrap()).to_vec();
        let bytes = Array::new();
        bytes.push(&js_sys::Uint8Array::from(&u8_array[..]));
        let blob = web_sys::Blob::new_with_u8_array_sequence(&bytes)?;
        js_blob_parts.push(&blob);
        cursor.continue_()?;

        Ok(())
    }
}

impl Drop for DownloadManager {
    fn drop(&mut self) {
        if self.idb.borrow().is_some() {
            let _ = self.idb.borrow().as_ref().unwrap().close();
        }
    }
}