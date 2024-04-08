use std::rc::Rc;
use log::{info, Level, warn};
use parking_lot::RwLock;

use web_sys::{CssStyleDeclaration};
use wgpu::{Adapter, Device, Features, Instance, Limits, Queue, RequestAdapterOptions};
use winit::dpi::{PhysicalSize};
use winit::event::{DeviceEvent, Event, TouchPhase, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use crate::device::device_state::DeviceState;
use crate::device::message_controller::{MessageController, SMEvent};
use crate::device::window_state::WindowState;




#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

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

    let (instance, adapter, device, queue) = {
        let mut limits: Limits = Limits::default();
        limits.max_buffer_size = (134217728) * 16;//128*20=2560 MB //WASM ONLY 2 Gb
        limits.max_storage_buffer_binding_size= (134217728) * 16-4; //(134217728) * 16;
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
        let _adapter: Adapter = _instance.request_adapter(&adapter_options).await.unwrap_or_else(|| { panic!("Cant init adapter") });

        let is_multi_draw_indirect = _adapter.features().contains(Features::MULTI_DRAW_INDIRECT);
        let (_device, _queue): (Device, Queue) = {
            if is_multi_draw_indirect {
                let (device, queue) = _adapter.request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: Features::MULTI_DRAW_INDIRECT,
                        required_limits: limits,
                    },
                    None,
                ).await.unwrap_or_else(|_| panic!("Cant init device"));
                ;
                (device, queue)
            } else {
                let (device, queue) = _adapter.request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: Features::default(),
                        required_limits: limits,
                    },
                    None,
                ).await.unwrap_or_else(|_| panic!("Cant init queue"));
                ;
                (device, queue)
            }
        };

        let max_b_size=_device.limits().max_buffer_size;
        let max_bin_size=_device.limits().max_storage_buffer_binding_size;
        warn!("SIZES {} {}",max_b_size,max_bin_size);

        let instance: Rc<RwLock<Instance>> = Rc::new(RwLock::new(_instance));
        let adapter: Rc<RwLock<Adapter>> = Rc::new(RwLock::new(_adapter));
        let device: Rc<RwLock<Device>> = Rc::new(RwLock::new(_device));
        let queue: Rc<RwLock<Queue>> = Rc::new(RwLock::new(_queue));
        (instance, adapter, device, queue)
    };
    let event_loop: EventLoop<()> = EventLoop::new().unwrap();

    let (window, canvas) = match web_sys::window() {
        None => { panic!("Cant WWASM WINDOW") }
        Some(win) => {
            use winit::platform::web::WindowExtWebSys;
            match win.document() {
                None => { panic!("Cant GET DOC") }
                Some(doc) => {
                    match doc.get_element_by_id("wasm3dwindow") {
                        None => { panic!("NO ID wasm3dwindow") }
                        Some(dst) => {
                            let sw = dst.client_width();
                            let sh = dst.client_height();
                            info!("HTML ROOM SIZE IS {} {}",sw,sh);
                            let ws: PhysicalSize<u32> = PhysicalSize::new(sw as u32, sh as u32);
                            //info!("HTML ROOM SIZE IS {} {}",sw,sh);
                            match WindowBuilder::new().with_inner_size(ws).build(&event_loop) {
                                Ok(window) => {
                                    let _scale_factor = window.scale_factor() as f32;
                                    info!("window ROOM SIZE IS {} {}",window.inner_size().width,window.inner_size().height);
                                    match window.canvas() {
                                        None => { panic!("CANT GET CANVAS") }
                                        Some(canvas) => {
                                            canvas.set_id("cws_main");
                                            let canvas_style: CssStyleDeclaration = canvas.style();
                                            let _ = canvas_style.set_property_with_priority("width", "99%", "");
                                            let _ = canvas_style.set_property_with_priority("height", "99%", "");
                                            match dst.append_child(&canvas).ok() {
                                                None => { panic!("CANT ATTACH CANVAS") }
                                                Some(_n) => {
                                                    warn! {"ATTACHED CANVAS SIZE is :{} {}",canvas.client_width(),canvas.client_height()}
                                                    let _sw = &canvas.client_width();
                                                    let _sh = &canvas.client_height();


                                                    //let ctx=canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
                                                    //ctx.scale(1.25,1.25);

                                                    (window, canvas)
                                                }
                                            }
                                            //(window, sw, sh, canvas)
                                        }
                                    }
                                    // (window, sw, sh)
                                }
                                Err(e) => { panic!("CANT BUILD WINDOWS {:?}", e) }
                            }
                        }
                    }
                }
            }
        }
    };

    let _window_state: Rc<RwLock<WindowState>> = Rc::new(RwLock::new(WindowState::new(
        window,
        instance.clone(),
        adapter.clone(),
        device.clone(),
        Some(canvas))));
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

    let mut is_mouse_active: bool = false;

    let window_state = _window_state.clone();
    let mc = message_controller.clone();


    event_loop.run(move |_event, elwt| {
        if !window_state.read().is_minimized() {
            mc.write().on_render();
            match &_event {
                Event::NewEvents(_start_cause) => {}
                Event::WindowEvent { window_id: _, event } => {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            window_state.write().resize(&physical_size, device.clone());
                            let sf = window_state.read().get_scale_factor();
                            warn!("RESIZE {} {} {}", physical_size.width, physical_size.height, sf);
                            mc.write().on_resize(physical_size.width, physical_size.height,sf);
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
                            if is_mouse_active {
                                match message_controller.clone().write().get_sender().try_send(SMEvent::KeyBoardEvent((device_id.clone(), event.clone(), *is_synthetic))) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        info!("Cant request KeyboardInput {:?}",e);
                                    }
                                }
                            }
                        }
                        WindowEvent::ModifiersChanged(_m) => {
                            //warn!("{:?}",m);
                        }
                        WindowEvent::Ime(_) => {}
                        WindowEvent::CursorMoved { device_id, position } => {
                            if is_mouse_active {
                                //mc.write().scene_state.camera.on_mouse(device_id.clone(), position.clone());
                                mc.write().on_mouse_move(device_id.clone(), position.clone());
                            }
                        }
                        WindowEvent::CursorEntered { device_id: _ } => {
                            is_mouse_active = true;
                        }
                        WindowEvent::CursorLeft { device_id: _ } => {
                            is_mouse_active = false;
                            //mc.write().scene_state.camera.relese_mouse();
                        }
                        WindowEvent::MouseWheel { device_id: _, delta: _, phase: _ } => {}
                        WindowEvent::MouseInput { device_id, state, button } => {
                            if is_mouse_active {
                                match message_controller.clone().write().get_sender().try_send(SMEvent::MouseButtonEvent((device_id.clone(), state.clone(), button.clone()))) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        info!("Cant request KeyboardInput {:?}",e);
                                    }
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
                                device_state.clone().as_ref().write().render(&window_state);
                            }

                            if mc.read().is_capture_screen_requested {
                                device_state.clone().as_ref().write().capture_screen(&window_state);
                                mc.write().is_capture_screen_requested=false;
                            }

                            //device_state.clone().as_ref().write().render(&window_state);
                        }
                        WindowEvent::ActivationTokenDone { .. } => {}
                    }
                }
                Event::DeviceEvent { device_id, event } => {
                    match event {
                        DeviceEvent::Added => {}
                        DeviceEvent::Removed => {}
                        DeviceEvent::MouseMotion { delta: _ } => {}
                        DeviceEvent::MouseWheel { delta } => {
                            if is_mouse_active {
                                mc.write().on_zoom(device_id.clone(), delta.clone(),  TouchPhase::Started);
                            }
                        }
                        DeviceEvent::Motion { axis: _, value: _ } => {}
                        DeviceEvent::Button { button: _, state: _ } => {}
                        DeviceEvent::Key(_key) => {}
                    }
                }
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
    }).expect("TODO: panic message")
}