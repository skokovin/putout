

use std::rc::Rc;
use cgmath::num_traits::Float;
use env_logger::{Builder, Target};
use log::{info, LevelFilter, warn};
use parking_lot::{RawRwLock, RwLock};
use parking_lot::lock_api::RwLockWriteGuard;

use wgpu::{Adapter, Device, Features, Instance, Limits, Queue, RequestAdapterOptions};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;

use winit::window::WindowBuilder;
use crate::device::device_state::DeviceState;
use crate::device::message_controller::{MessageController, SMEvent};
use crate::device::window_state::WindowState;
use crate::gui::camera_base::CameraMode;

#[cfg(not(target_arch = "wasm32"))]
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
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

    let (instance, adapter, device, queue) = {
        let mut limits: Limits = Limits::default();
        limits.max_buffer_size = (134217728) * 20;//128*20=2560 MB
        limits.max_storage_buffer_binding_size= (134217728) * 16;
        let _instance: Instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: Default::default(),
        });
        let adapter_options = RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        };
        let _adapter: Adapter = pollster::block_on(_instance.request_adapter(&adapter_options)).unwrap();
        let is_multi_draw_indirect = _adapter.features().contains(Features::MULTI_DRAW_INDIRECT);
        let (_device, _queue): (Device, Queue) = {
            if is_multi_draw_indirect {
                let (device, queue) = pollster::block_on(_adapter.request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: Features::MULTI_DRAW_INDIRECT,
                        required_limits: limits,
                    },
                    None,
                )).unwrap();
                (device, queue)
            } else {
                let (device, queue) = pollster::block_on(_adapter.request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: Features::default(),
                        required_limits: limits,
                    },
                    None,
                )).unwrap();
                (device, queue)
            }
        };

        let instance: Rc<RwLock<Instance>> = Rc::new(RwLock::new(_instance));
        let adapter: Rc<RwLock<Adapter>> = Rc::new(RwLock::new(_adapter));
        let device: Rc<RwLock<Device>> = Rc::new(RwLock::new(_device));
        let queue: Rc<RwLock<Queue>> = Rc::new(RwLock::new(_queue));
        (instance, adapter, device, queue)
    };


    let event_loop: EventLoop<()> = EventLoop::new().unwrap();

    let _window_state: Rc<RwLock<WindowState>> = Rc::new(RwLock::new(WindowState::new(
        WindowBuilder::new().with_min_inner_size(PhysicalSize::new(800, 600)).with_inner_size(PhysicalSize::new(800, 600)).build(&event_loop).unwrap(),
        instance.clone(),
        adapter.clone(),
        device.clone(),
        queue.clone(),
        None)));


    let text_layout:  Rc<RwLock<TextLayout>> =Rc::new(RwLock::new(TextLayout::new(device.clone(), queue.clone(), _window_state.read().config.clone())));

    let message_controller: Rc<RwLock<MessageController>> = Rc::new(RwLock::new(MessageController::new(device.clone(), _window_state.clone(),text_layout.clone())));

    let device_state: Rc<RwLock<DeviceState>> = Rc::new(RwLock::new(
        DeviceState::new(
            message_controller.clone(),
            instance.clone(),
            adapter.clone(),
            device.clone(),
            queue.clone(),
        )
    ));


    let window_state: Rc<RwLock<WindowState>> = _window_state.clone();
    let mc: Rc<RwLock<MessageController>> = message_controller.clone();
    event_loop.run(move |_event, elwt| {
        if !window_state.read().is_minimized() {
            mc.write().on_render();
            match &_event {
                Event::NewEvents(_start_cause) => {}
                Event::WindowEvent { window_id: _, event } => {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            window_state.write().resize(&physical_size, device.clone());
                            let sf: f64 =window_state.read().get_scale_factor();
                            warn!("RESIZE {} {} {}", physical_size.width, physical_size.height, sf);
                            mc.write().on_resize(physical_size.width, physical_size.height, sf);
                        }
                        WindowEvent::Moved(_) => {}
                        WindowEvent::CloseRequested => {
                            elwt.exit();
                        }
                        WindowEvent::Destroyed => {}
                        WindowEvent::DroppedFile(_) => {}
                        WindowEvent::HoveredFile(_) => {}
                        WindowEvent::HoveredFileCancelled => {}
                        WindowEvent::Focused(_event) => {}
                        WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {


                            match message_controller.clone().write().get_sender().try_send(SMEvent::KeyBoardEvent((device_id.clone(), event.clone(), *is_synthetic))) {
                                Ok(_) => {}
                                Err(e) => {
                                    info!("Cant request KeyboardInput {:?}",e);
                                }
                            }
                        }
                        WindowEvent::ModifiersChanged(_m) => {
                            //warn!("{:?}",m);
                        }
                        WindowEvent::Ime(_) => {}
                        WindowEvent::CursorMoved { device_id, position } => {

                            let state = mc.read().scene_state.camera.mode;

                            match state {
                                CameraMode::FLY => {
                                    let wsize = window_state.read().get_window_size();
                                    let _sf = window_state.read().get_scale_factor() as f32;

                                    if position.x < f64::epsilon() {
                                        window_state.write().set_cursor_position(PhysicalPosition::new(wsize.width, position.y as f32));
                                        mc.write().scene_state.camera.relese_mouse();
                                    } else if (wsize.width as f32 - position.x as f32 - 1.0) < f32::epsilon() {
                                        window_state.write().set_cursor_position(PhysicalPosition::new(0.0, position.y as f32));
                                        mc.write().scene_state.camera.relese_mouse();
                                    }

                                    if position.y < f64::epsilon() {
                                        window_state.write().set_cursor_position(PhysicalPosition::new(position.x as f32, wsize.height));
                                        mc.write().scene_state.camera.relese_mouse();
                                    } else if (wsize.height as f32 - position.y as f32 - 1.0) < f32::epsilon() {
                                        window_state.write().set_cursor_position(PhysicalPosition::new(position.x as f32, 0.0));
                                        mc.write().scene_state.camera.relese_mouse();
                                    }
                                    mc.write().scene_state.camera.on_mouse(device_id.clone(), position.clone());
                                }
                                CameraMode::ORBIT => {
                                    mc.write().on_mouse_move(device_id.clone(), position.clone());
                                }
                                CameraMode::TOUCH => {}
                            }
                        }
                        WindowEvent::CursorEntered { device_id: _ } => {}
                        WindowEvent::CursorLeft { device_id: _ } => {
                            mc.write().scene_state.camera.relese_mouse();
                        }
                        WindowEvent::MouseWheel { device_id, delta, phase } => {
                            mc.write().on_zoom(device_id.clone(), delta.clone(), phase.clone());
                        }
                        WindowEvent::MouseInput { device_id, state, button } => {
                            match message_controller.clone().write().get_sender().try_send(SMEvent::MouseButtonEvent((device_id.clone(), state.clone(),button.clone()))) {
                                Ok(_) => {}
                                Err(e) => {
                                    info!("Cant request KeyboardInput {:?}",e);
                                }
                            }

                        }
                        WindowEvent::TouchpadMagnify { .. } => {}
                        WindowEvent::SmartMagnify { .. } => {}
                        WindowEvent::TouchpadRotate { .. } => {}
                        WindowEvent::TouchpadPressure { .. } => {}
                        WindowEvent::AxisMotion { .. } => {}
                        WindowEvent::Touch(_0) => {}
                        WindowEvent::ScaleFactorChanged { .. } => {}
                        WindowEvent::ThemeChanged(_) => {}
                        WindowEvent::Occluded(_) => {}
                        WindowEvent::RedrawRequested => {
                            {
                                device_state.clone().as_ref().write().render(_window_state.clone());
                            }
                          

                            if mc.read().is_capture_screen_requested {
                                device_state.clone().as_ref().write().capture_screen(&window_state);
                                mc.write().is_capture_screen_requested=false;
                            }
                        }
                        WindowEvent::ActivationTokenDone { .. } => {}
                    }
                }
                Event::DeviceEvent { device_id: _, event: _ } => {}
                Event::UserEvent(_ue) => {}
                Event::Suspended => {}
                Event::Resumed => {}
                Event::AboutToWait => {
                    window_state.write().request_redraw(device_state.clone().as_ref());
                }
                Event::LoopExiting => {}
                Event::MemoryWarning => {}
            }
        }
    }).expect("TODO: panic message");
}
