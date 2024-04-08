
use std::sync::Mutex;
use log::{Level, warn};
use miniz_oxide::inflate::decompress_to_vec;
use once_cell::sync::Lazy;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::js_sys::{Float32Array,Uint8Array};

use crate::device::message_controller::SnapMode;
use crate::remote::{ArrayF32State, CommandState, RemoteCommand, RemoteMeshData};



pub static REMOTE_HULL_MESH: Lazy<Mutex<RemoteMeshData>> = Lazy::new(|| Mutex::new(RemoteMeshData::new()));
pub static DIMENSIONING: Lazy<Mutex<SnapMode>> = Lazy::new(|| Mutex::new(SnapMode::Disabled));
pub static SLICER: Lazy<Mutex<ArrayF32State>> = Lazy::new(|| Mutex::new(ArrayF32State::new()));
pub static COMMANDS: Lazy<Mutex<CommandState>> = Lazy::new(|| Mutex::new(CommandState::new()));

pub unsafe fn debug_move_to(){
    match COMMANDS.lock() {
        Ok(mut m) => {
            //1904245 17193573
            m.values.push_back(RemoteCommand::MoveCameraToOID(1904245));
        }
        Err(_e) => { warn!("CANT LOCK changeSlicer MEM") }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async unsafe fn wasm_changeSlicer(planes: Float32Array) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(Level::Info);
    let planes: Vec<f32> = planes.to_vec();
    match SLICER.lock() {
        Ok(mut m) => {
            m.values.clear();
            m.values.extend(&planes);
            m.is_dirty = true;
        }
        Err(_e) => { warn!("CANT LOCK changeSlicer MEM") }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async unsafe fn wasm_movecamtostartpos() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(Level::Info);

    match COMMANDS.lock() {
        Ok(mut m) => {
            m.values.push_back(RemoteCommand::MoveCameraToStartPos);
        }
        Err(_e) => { warn!("CANT LOCK changeSlicer MEM") }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async unsafe fn wasm_movecamtooid(oid:i32) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(Level::Info);
    match COMMANDS.lock() {
        Ok(mut m) => {
            m.values.push_back(RemoteCommand::MoveCameraToOID(oid));
        }
        Err(_e) => { warn!("CANT LOCK changeSlicer MEM") }
    }
}
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async unsafe fn enable_dimensioning(mode:i32) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(Level::Info);
    match DIMENSIONING.lock() {

        Ok(mut curr_value) => {
            match mode {
                mode if mode == SnapMode::Vertex as i32 => {
                    *curr_value=SnapMode::Vertex;
                    warn!("MODE IS {}",mode)
                },
                mode if mode == SnapMode::Edge as i32 => {
                    *curr_value=SnapMode::Edge;
                    warn!("MODE IS {}",mode)
                },
                mode if mode ==  SnapMode::Disabled as i32 =>{
                    *curr_value=SnapMode::Disabled;
                    warn!("MODE IS {}",mode)
                },
                _ => {

                }
            }
        }
        Err(_e) => { warn!("CANT LOCK DIMENSIONING MEM") }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async unsafe fn wasm_unpack_hull(arr_v: Uint8Array, arr_i: Uint8Array, arr_b: Uint8Array, arr_t: Uint8Array)->bool {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(Level::Warn);
    warn!("TRY UNPACK V {}", arr_v.length());
    let handler_v: Vec<u8> = arr_v.to_vec();
    let decoded_v: Vec<u8> = decompress_to_vec(&handler_v).unwrap();
    warn!("TRY UNPACK I {}", arr_i.length());
    let handler_i: Vec<u8> = arr_i.to_vec();
    let decoded_i: Vec<u8> = decompress_to_vec(&handler_i).unwrap();
    warn!("TRY UNPACK B {}", arr_b.length());
    let handler_b: Vec<u8> = arr_b.to_vec();
    let decoded_b: Vec<u8> = decompress_to_vec(&handler_b).unwrap();

    warn!("TRY UNPACK T {}", arr_t.length());
    let handler_t: Vec<u8> = arr_t.to_vec();
    let decoded_t: Vec<u8> = decompress_to_vec(&handler_t).unwrap();


    //mesh_loader::read_unpacked_wasm(decoded_v, decoded_i, decoded_b, decoded_t);


    //let mut hull_mesh: Vec<RawMesh> = mesh_loader::read_unpacked(decoded_v, decoded_i, decoded_b, decoded_t);

    //warn!("UNPACKED HULL MESHES {}", hull_mesh.len());

    match REMOTE_HULL_MESH.lock() {
        Ok(mut m) => {
            m.clean();
            m.is_dirty = true;
            m.decoded_v.extend_from_slice(decoded_v.as_slice());
            m.decoded_i.extend_from_slice(decoded_i.as_slice());
            m.decoded_b.extend_from_slice(decoded_b.as_slice());
            m.decoded_t.extend_from_slice(decoded_t.as_slice());
        }
        Err(_e) => { warn!("CANT LOCK REMOTE_HULL_MESH MEM") }
    }
    true
}



