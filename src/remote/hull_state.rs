use std::collections::HashSet;
use std::sync::Mutex;

use log::{Level, warn};
use once_cell::sync::Lazy;
use crate::remote::HashI32State;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::js_sys::{Float32Array, Int32Array};
use wasm_bindgen_futures::js_sys::Uint8Array;


pub static SELECTED_HULL: Lazy<Mutex<HashI32State>> = Lazy::new(|| Mutex::new(HashI32State::new()));

pub static HIDDEN_HULL: Lazy<Mutex<HashI32State>> = Lazy::new(|| Mutex::new(HashI32State::new()));

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async unsafe fn hull_add_selected(_ids: Int32Array) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(Level::Info);
    let ids: HashSet<i32> = _ids.to_vec().into_iter().collect();

    match SELECTED_HULL.lock() {
        Ok(mut m) => {
            m.values.clear();
            m.values.extend(&ids);
            m.is_dirty = true;
            warn!("SELECTED {} ADDED, TOTAL {}",ids.len(),m.values.len());
        }
        Err(_e) => { warn!("CANT LOCK SELECTED SHARED MEM") }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async unsafe fn hull_add_hidden(_ids: Int32Array) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(Level::Info);
    let ids: Vec<i32> = _ids.to_vec().into_iter().collect();
    match HIDDEN_HULL.lock() {
        Ok(mut m) => {
            m.values.extend(&ids);
            m.is_dirty = true;
            warn!("HIDDEN {} ADDED, TOTAL {}",ids.len(),m.values.len());
        }
        Err(_e) => { warn!("CANT LOCK HIDDEN SHARED MEM") }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async unsafe fn hull_clear_selected() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(Level::Info);
    match SELECTED_HULL.lock() {
        Ok(mut m) => {
            m.values.clear();
            m.is_dirty = true;
            warn!("SELECTED CLEARED");
        }
        Err(_e) => { warn!("CANT LOCK SELECTED SHARED MEM") }
    }
}


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async unsafe fn hull_clear_hidden() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(Level::Info);
    match HIDDEN_HULL.lock() {
        Ok(mut m) => {
            m.values.clear();
            m.is_dirty = true;
            warn!("SELECTED CLEARED");
        }
        Err(_e) => { warn!("CANT LOCK SELECTED SHARED MEM") }
    }
}


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = wvservice)]
    pub fn log2();

    #[wasm_bindgen(js_namespace = wvservice)]
    pub fn get_vertex_array(id:i32)->Uint8Array;
    #[wasm_bindgen(js_namespace = wvservice)]
    pub fn get_index_array(id:i32)->Uint8Array;
    #[wasm_bindgen(js_namespace = wvservice)]
    pub fn get_bbx_array(id:i32)->Uint8Array;
    #[wasm_bindgen(js_namespace = wvservice)]
    pub fn get_types_array(id:i32)->Uint8Array;

    #[wasm_bindgen(js_namespace = wvservice)]
    pub fn select_hull_parts_remote(ids: Int32Array);

    #[wasm_bindgen(js_namespace = wvservice)]
    pub fn hide_hull_parts_remote(ids: Int32Array);
    #[wasm_bindgen(js_namespace = wvservice)]
    pub fn dim_set_fist_point(coords: Float32Array);
    #[wasm_bindgen(js_namespace = wvservice)]
    pub fn dim_set_second_point(coords: Float32Array);
}

