use std::rc::Rc;
use std::sync::TryLockResult;
use cgmath::num_traits::real::Real;
use log::{info, warn};
use parking_lot::RwLock;
#[cfg(not(target_arch = "wasm32"))]
use wgpu::{Adapter, Device, Features, Instance, Limits, Queue, RequestAdapterOptions};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceEvent, DeviceId, TouchPhase, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::device::device_state::DeviceState;
use crate::device::message_controller::{MessageController};
use crate::device::window_state::WindowState;
use crate::gui::camera_base::CameraMode;
use crate::remote::common_state::COMMANDS;
use crate::remote::RemoteCommand;
use crate::shared::text_layout::TextLayout;

#[cfg(not(target_arch = "wasm32"))]
pub struct MState {
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

#[cfg(not(target_arch = "wasm32"))]
impl MState {
    pub fn new() -> Self {
        let (instance, adapter, device, queue): (Rc<RwLock<Instance>>, Rc<RwLock<Adapter>>, Rc<RwLock<Device>>, Rc<RwLock<Queue>>) = {
            let mut limits: Limits = Limits::default();
            limits.max_buffer_size = (134217728) * 20;//128*20=2560 MB
            limits.max_storage_buffer_binding_size = (134217728) * 16;
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

#[cfg(not(target_arch = "wasm32"))]
impl ApplicationHandler for MState {
    // This is a common indicator that you can create a window.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attr = Window::default_attributes().with_min_inner_size(PhysicalSize::new(800, 600)).with_inner_size(PhysicalSize::new(800, 600));
        let w = Rc::new(RwLock::new(event_loop.create_window(attr).unwrap()));
        self.window = Some(w.clone());
        let _window_state: Rc<RwLock<WindowState>> = Rc::new(RwLock::new(WindowState::new(
            w.clone(),
            self.instance.clone(),
            self.adapter.clone(),
            self.device.clone(),
            self.queue.clone(),
            None)));

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
        let window = self.window.as_ref().unwrap();
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
                match COMMANDS.try_lock() {
                    Ok(mut m) => {
                        m.values.push_back(RemoteCommand::OnKeyBoard((device_id, event, is_synthetic)));
                    }
                    Err(_e) => { warn!("CANT LOCK COMMANDS MEM") }
                }
            }
            WindowEvent::ModifiersChanged(_) => {}
            WindowEvent::Ime(_) => {}
            WindowEvent::CursorMoved { device_id, position } => {
               let state = self.message_controller.as_ref().unwrap().read().scene_state.camera.mode;
                match state {
                    CameraMode::FLY => {}
                    CameraMode::ORBIT => {
                        match COMMANDS.try_lock() {
                            Ok(mut m) => {
                                m.values.push_back(RemoteCommand::OnMouseMove((device_id, position)));
                            }
                            Err(_e) => { warn!("CANT LOCK COMMANDS MEM") }
                        }
                    }
                    CameraMode::TOUCH => {}
                }
            }
            WindowEvent::CursorEntered { .. } => {}
            WindowEvent::CursorLeft { device_id } => {
                self.message_controller.as_ref().unwrap().write().scene_state.camera.relese_mouse();
            }
            WindowEvent::MouseWheel { device_id, delta, phase } => {
                //self.message_controller.as_ref().unwrap().write().on_zoom(device_id.clone(), delta.clone(), phase.clone());
                match COMMANDS.try_lock() {
                    Ok(mut m) => {
                        m.values.push_back(RemoteCommand::OnMouseWheel((device_id, delta, phase)));
                        //window.read().request_redraw();
                    }
                    Err(_e) => { warn!("CANT LOCK COMMANDS MEM") }
                }
            }
            WindowEvent::MouseInput { device_id, state, button } => {
                match COMMANDS.try_lock() {
                    Ok(mut m) => {
                        m.values.push_back(RemoteCommand::OnMouseButton((device_id, state, button)));
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
                if (
                    self.message_controller.as_ref().unwrap().read().is_capture_screen_requested &&
                        !self.message_controller.as_ref().unwrap().read().is_mouse_btn_active
                ) {
                    self.device_state.as_ref().unwrap().write().capture_screen(self.window_state.as_ref().unwrap());
                    self.message_controller.as_ref().unwrap().write().is_capture_screen_requested = false;
                }
                self.device_state.as_ref().unwrap().clone().as_ref().write().render(self.window_state.as_ref().unwrap().clone());
                //window.read().request_redraw();
            }
        }
    }
    fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {
        let window = self.window.as_ref().unwrap();
        match event {
            DeviceEvent::Added => {}
            DeviceEvent::Removed => {}
            DeviceEvent::MouseMotion { delta } => {
                let state = self.message_controller.as_ref().unwrap().read().scene_state.camera.mode;
                match state {
                    CameraMode::FLY => {
                        self.message_controller.as_ref().unwrap().write().scene_state.camera.on_mouse_dx_dy(device_id.clone(), delta.0, delta.1);
                        window.read().request_redraw();
                    }
                    CameraMode::ORBIT => {
                        self.message_controller.as_ref().unwrap().write().scene_state.camera.on_mouse_dx_dy(device_id.clone(), delta.0, delta.1);
                        window.read().request_redraw();
                        //self.message_controller.as_ref().unwrap().write().on_mouse_move_delta(device_id.clone(), delta.0, delta.1);
               /*         match COMMANDS.lock() {
                            Ok(mut m) => {
                                m.values.push_back(RemoteCommand::OnMouseMoveDelta((device_id, delta.0, delta.1)));
                            }
                            Err(_e) => { warn!("CANT LOCK COMMANDS MEM") }
                        }*/
                    }
                    CameraMode::TOUCH => {}
                }

            }
            DeviceEvent::MouseWheel { delta } => {
                //self.message_controller.as_ref().unwrap().write().on_zoom(device_id.clone(), delta.clone(), TouchPhase::Ended);
            }
            DeviceEvent::Motion { .. } => {}
            DeviceEvent::Button { .. } => {}
            DeviceEvent::Key(_) => {}
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let window = self.window.as_ref().unwrap();
        window.read().request_redraw();
        //self.window_state.as_ref().unwrap().write().request_redraw(self.device_state.as_ref().unwrap().as_ref())
    }
}