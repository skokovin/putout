use std::rc::Rc;
use log::{info, Level, warn};
use parking_lot::RwLock;


use wgpu::{Adapter, Device, Instance, Limits, Queue, RequestAdapterOptions};
use winit::dpi::{PhysicalSize};
use winit::event::{DeviceEvent, Event, TouchPhase, WindowEvent};
use winit::event_loop::EventLoop;

use crate::device::device_state::DeviceState;
use crate::device::message_controller::{MessageController};
use crate::device::window_state::WindowState;


#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(target_arch = "wasm32")]
use crate::device::wstate::WState;

use crate::shared::text_layout::TextLayout;

mod device;
mod shared;
mod gui;
mod scene;
mod remote;

#[wasm_bindgen]
#[cfg(target_arch = "wasm32")]
pub async fn runrust() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(Level::Info);
    let event_loop = EventLoop::new().unwrap();
    info!("PRE1");
    let mut state = WState::new().await;
    info!("PRE2");
    let _ = event_loop.run_app(&mut state);
    println!("out")
}


