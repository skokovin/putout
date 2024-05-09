use std::rc::Rc;
use cgmath::num_traits::Float;

use log::{info, warn};
use parking_lot::RwLock;

use wgpu::{Adapter, Backends, Device, DeviceDescriptor, Instance, Limits, Queue, RequestAdapterOptions};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::error::OsError;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;
use winit::window::{Window, WindowId};
use crate::device::device_state::DeviceState;
use crate::device::message_controller::{MessageController};
use crate::device::window_state::WindowState;
use crate::gui::camera_base::CameraMode;
use crate::remote::common_state::COMMANDS;
use crate::remote::RemoteCommand;
use crate::shared::text_layout::TextLayout;

pub struct WState {
    window: Option<Rc<RwLock<Window>>>,
    instance: Rc<RwLock<Instance>>,
    adapter: Rc<RwLock<Adapter>>,
    device: Rc<RwLock<Device>>,
    queue: Rc<RwLock<Queue>>,
    window_state: Option<Rc<RwLock<WindowState>>>,
    message_controller: Option<Rc<RwLock<MessageController>>>,
    device_state: Option<Rc<RwLock<DeviceState>>>,
    counter: i32,
}

impl WState {
    pub async fn new() -> Self {
        let (instance, adapter, device, queue): (Rc<RwLock<Instance>>, Rc<RwLock<Adapter>>, Rc<RwLock<Device>>, Rc<RwLock<Queue>>) = {
            let mut limits: Limits = Limits::default();
            limits.max_buffer_size = (134217728) * 16;//128*20=2560 MB //WASM ONLY 2 Gb
            limits.max_storage_buffer_binding_size= (134217728) * 16-4; //(134217728) * 16;
            let _instance: Instance = Instance::new(wgpu::InstanceDescriptor {
                backends: Backends::PRIMARY,
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
            let mut dd=DeviceDescriptor::default();
            dd.required_limits=limits;

            let (_device, _queue): (Device, Queue) = {
                let (device, queue) = _adapter.request_device(
                    &dd,
                    None,
                ).await.unwrap_or_else(|e| panic!("Cant init queue {:?}",e));
                (device, queue)
            };


            let instance: Rc<RwLock<Instance>> = Rc::new(RwLock::new(_instance));
            let adapter: Rc<RwLock<Adapter>> = Rc::new(RwLock::new(_adapter));
            let device: Rc<RwLock<Device>> = Rc::new(RwLock::new(_device));
            let queue: Rc<RwLock<Queue>> = Rc::new(RwLock::new(_queue));
            (instance, adapter, device, queue)
        };

        Self {
            window: None,
            instance: instance,
            adapter: adapter,
            device: device,
            queue: queue,
            window_state: None,
            message_controller: None,
            device_state: None,
            counter: 0,
        }
    }
}


impl ApplicationHandler for WState {
    #[cfg(not(target_arch = "wasm32"))]
    fn resumed(&mut self, event_loop: &ActiveEventLoop){}
    // This is a common indicator that you can create a window.
    #[cfg(target_arch = "wasm32")]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, canvas) = match web_sys::window() {
            None => { panic!("Cant WWASM WINDOW") }
            Some(win) => {
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
                                let attr = Window::default_attributes().with_inner_size(ws);
                                match event_loop.create_window(attr){
                                    Ok(window) => {
                                        let _scale_factor = window.scale_factor() as f32;
                                        match window.canvas() {
                                            None => {panic!("CANT GET CANVAS")}
                                            Some(canvas) => {
                                                canvas.set_id("cws_main");
                                                let canvas_style = canvas.style();
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
                                    }
                                    Err(e) => {panic!("CANT BUILD WINDOWS {:?}", e)}
                                }

                                //info!("HTML ROOM SIZE IS {} {}",sw,sh);

                            }
                        }
                    }
                }
            }
        };


        let w = Rc::new(RwLock::new(window));
        self.window = Some(w.clone());
        let _window_state: Rc<RwLock<WindowState>> = Rc::new(RwLock::new(WindowState::new(
            w.clone(),
            self.instance.clone(),
            self.adapter.clone(),
            self.device.clone(),
            self.queue.clone(),
            Some(canvas))));

        let text_layout: Rc<RwLock<TextLayout>> = Rc::new(RwLock::new(TextLayout::new(self.device.clone(), self.queue.clone(), _window_state.read().config.clone())));
        let message_controller: Rc<RwLock<MessageController>> = Rc::new(RwLock::new(MessageController::new(self.device.clone(), _window_state.clone(), text_layout.clone())));

        let device_state: Rc<RwLock<DeviceState>> = Rc::new(RwLock::new(
            DeviceState::new(
                message_controller.clone(),
                self.instance.clone(),
                self.adapter.clone(),
                self.device.clone(),
                self.queue.clone(),
            )
        ));
        self.message_controller = Some(message_controller);
        self.device_state = Some(device_state);
        self.window_state = Some(_window_state);
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        // `unwrap` is fine, the window will always be available when
        // receiving a window event.
        //let window = self.window.as_ref().unwrap();
        self.message_controller.as_ref().unwrap().write().on_render();
        match event {
            WindowEvent::ActivationTokenDone { .. } => {}
            WindowEvent::Resized(physical_size) => {
                self.window_state.as_ref().unwrap().write().resize(&physical_size, self.device.clone());
                let sf: f64 = self.window_state.as_ref().unwrap().read().get_scale_factor();
                self.message_controller.as_ref().unwrap().write().on_resize(physical_size.width, physical_size.height, sf);
            }
            WindowEvent::Moved(_) => {}
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Destroyed => {}
            WindowEvent::DroppedFile(_) => {}
            WindowEvent::HoveredFile(_) => {}
            WindowEvent::HoveredFileCancelled => {}
            WindowEvent::Focused(_) => {}
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                match COMMANDS.lock() {
                    Ok(mut m) => {
                        m.values.push_back(RemoteCommand::OnKeyBoard((device_id, event, is_synthetic)));
                    }
                    Err(_e) => { warn!("CANT LOCK COMMANDS MEM") }
                }
            }
            WindowEvent::ModifiersChanged(_) => {}
            WindowEvent::Ime(_) => {}
            WindowEvent::CursorMoved { device_id, position } => {

                let state =self.message_controller.as_ref().unwrap().read().scene_state.camera.mode;

                match state {
                    CameraMode::FLY => {}
                    CameraMode::ORBIT => {
                        self.message_controller.as_ref().unwrap().write().on_mouse_move(device_id.clone(), position.clone());
                    }
                    CameraMode::TOUCH => {}
                }

            }
            WindowEvent::CursorEntered { .. } => {}
            WindowEvent::CursorLeft { device_id } => {
                self.message_controller.as_ref().unwrap().write().scene_state.camera.relese_mouse();
            }
            WindowEvent::MouseWheel { device_id, delta, phase } => {
                self.message_controller.as_ref().unwrap().write().on_zoom(device_id.clone(), delta.clone(), phase.clone());
                self.message_controller.as_ref().unwrap().write().is_capture_screen_requested = true;
            }
            WindowEvent::MouseInput { device_id, state, button } => {
                match COMMANDS.lock() {
                    Ok(mut m) => {
                        m.values.push_back(RemoteCommand::OnMouseButton((device_id,state,button)));
                    }
                    Err(_e) => { warn!("CANT LOCK COMMANDS MEM") }
                }
            }
            WindowEvent::PinchGesture { .. } => {}
            WindowEvent::PanGesture { .. } => {}
            WindowEvent::DoubleTapGesture { .. } => {}
            WindowEvent::RotationGesture { .. } => {}
            WindowEvent::TouchpadPressure { .. } => {}
            WindowEvent::AxisMotion { .. } => {}
            WindowEvent::Touch(_) => {}
            WindowEvent::ScaleFactorChanged { .. } => {}
            WindowEvent::ThemeChanged(_) => {}
            WindowEvent::Occluded(_) => {}
            WindowEvent::RedrawRequested => {
                if let Some(window) = self.window.as_ref() {
                    {
                        self.device_state.as_ref().unwrap().clone().as_ref().write().render(self.window_state.as_ref().unwrap().clone());
                    }
                    if (
                        self.message_controller.as_ref().unwrap().read().is_capture_screen_requested
                            &&!self.message_controller.as_ref().unwrap().read().is_mouse_btn_active
                    ) {
                        self.device_state.as_ref().unwrap().write().capture_screen(self.window_state.as_ref().unwrap());
                        self.message_controller.as_ref().unwrap().write().is_capture_screen_requested = false;
                    }
                    window.read().request_redraw();
                }
            }
        }
    }
    fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::Added => {}
            DeviceEvent::Removed => {}
            DeviceEvent::MouseMotion { delta } => {
               let state = self.message_controller.as_ref().unwrap().read().scene_state.camera.mode;
                match state {
                    CameraMode::FLY => {
                        self.message_controller.as_ref().unwrap().write().scene_state.camera.on_mouse_dx_dy(device_id.clone() ,delta.0,delta.1);
                    }
                    CameraMode::ORBIT => {}
                    CameraMode::TOUCH => {}
                }
            }
            DeviceEvent::MouseWheel { .. } => {}
            DeviceEvent::Motion { .. } => {}
            DeviceEvent::Button { .. } => {}
            DeviceEvent::Key(_) => {}
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.window_state.as_ref().unwrap().write().request_redraw( self.device_state.as_ref().unwrap().as_ref());
    }
}