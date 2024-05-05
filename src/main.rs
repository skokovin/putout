use std::rc::Rc;
use cgmath::num_traits::Float;
use env_logger::{Builder, Target};
use log::{info, LevelFilter, warn};
use parking_lot::{RawRwLock, RwLock};
use parking_lot::lock_api::RwLockWriteGuard;
use serde_json::ser::State;

use wgpu::{Adapter, Device,  Instance, Limits, Queue, RequestAdapterOptions};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceEvent, DeviceId, Event, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};

use crate::device::device_state::DeviceState;
use crate::device::message_controller::{MessageController, SMEvent};
use crate::device::window_state::WindowState;
use crate::gui::camera_base::CameraMode;

#[cfg(not(target_arch = "wasm32"))]
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::window::{Window, WindowId};
#[cfg(not(target_arch = "wasm32"))]
use crate::device::state::MState;
use crate::shared::text_layout::TextLayout;

mod device;
mod shared;
mod gui;
mod scene;

mod remote;





fn main() {
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.filter(None, LevelFilter::Warn).init();


    let event_loop = EventLoop::new().unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    let mut state = MState::new();
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event_loop.run_app(&mut state);
    println!("out")
}

