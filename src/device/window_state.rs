use std::rc::Rc;
use log::{info, warn};
use parking_lot::RwLock;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::js_sys::Object;
use web_sys::{GpuCanvasConfiguration, GpuCanvasContext, HtmlCanvasElement, window};
use wgpu::{Adapter, Device, Instance, Surface, SurfaceCapabilities, SurfaceConfiguration, SurfaceError, SurfaceTexture, TextureFormat};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceId, KeyEvent};
use winit::window::{CursorGrabMode, Window};
use crate::device::device_state::DeviceState;
use crate::shared::highlight_pipeline::HighlightPipeLine;
use crate::shared::mesh_pipeline::MeshPipeLine;

#[derive(PartialEq)]
pub enum WindowMode {
    CursorVisible,
    CursorInVisible,
}

pub struct WindowState {
    window: Window,
    canvas: Option<HtmlCanvasElement>,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    pub mesh_pipeline: RwLock<MeshPipeLine>,
    pub highlight_pipeline: RwLock<HighlightPipeLine>,
    pub window_mode: WindowMode,
}

impl WindowState {
    pub fn new(window: Window, instance: Rc<RwLock<Instance>>, adapter: Rc<RwLock<Adapter>>, device: Rc<RwLock<Device>>, canvas: Option<HtmlCanvasElement>) -> Self {
        #[cfg(target_arch = "wasm32")]
            let size: PhysicalSize<u32> = {
            match &canvas {
                None => { panic!("Problem With Canvas") }
                Some(c) => {
                    info!("MYCANVAS {} {}",c.client_width(),c.client_height());
                    let html_w = c.client_width() as u32;
                    let html_h = c.client_height() as u32;
                    PhysicalSize::new(html_w, html_h)
                }
            }
        };

        #[cfg(not(target_arch = "wasm32"))]
            let size: PhysicalSize<u32> = window.inner_size();


        let surface = unsafe {
            match wgpu::SurfaceTargetUnsafe::from_window(&window) {
                Ok(st) => {
                    match instance.clone().write().create_surface_unsafe(st) {
                        Ok(s) => {
                            info! {"MyWindow is {} {}",size.width,size.height}
                            s
                        }
                        Err(e) => { panic!("THERE IS NO SURFACE {:?}", e); }
                    }
                }
                Err(e) => { panic!("THERE IS NO SurfaceTargetUnsafe {:?}", e); }
            }
        };
        let capabilities: SurfaceCapabilities = surface.get_capabilities(&adapter.clone().read());
        let format: TextureFormat = *capabilities.formats.first().expect("No supported texture formats.");
        let config: SurfaceConfiguration = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: format.clone(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 0,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device.clone().read(), &config);
        let mesh_pipeline: RwLock<MeshPipeLine> = RwLock::new(MeshPipeLine::new(device.clone(), format.clone()));
        let highlight_pipeline: RwLock<HighlightPipeLine> = RwLock::new(HighlightPipeLine::new(device.clone(), format.clone()));
        Self {
            window: window,
            canvas: canvas,
            surface: surface,
            config: config,
            mesh_pipeline: mesh_pipeline,
            highlight_pipeline: highlight_pipeline,
            window_mode: WindowMode::CursorVisible,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn resize(&mut self, size: &PhysicalSize<u32>, device: Rc<RwLock<Device>>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&device.clone().read(), &self.config);
    }
    #[cfg(target_arch = "wasm32")]
    pub fn resize(&mut self, size: &PhysicalSize<u32>,  device: Rc<RwLock<Device>>) {}
    pub fn request_redraw(&mut self, device_state: &RwLock<DeviceState>) {
        #[cfg(target_arch = "wasm32")]
        {
            match &mut self.canvas {
                None => {}
                Some(canvas) => {
                    let ctw = self.surface.get_current_texture().unwrap().texture.width();
                    let cth = self.surface.get_current_texture().unwrap().texture.height();
                    let cw = canvas.client_width() as u32;
                    let ch = canvas.client_height() as u32;


     /*               match canvas.get_context("webgpu") {
                        Ok(ctx2d) => {
                            match ctx2d {
                                None => {
                                    warn!("NO OBJ")
                                }
                                Some(obj) => {
                                   match obj.dyn_into::<web_sys::GpuCanvasContext>() {
                                       Ok(ctx3d) => {

                                       }
                                       Err(e) => {warn!("NO GpuCanvasContext")}
                                   }
                                }
                            }
                            //let ctx2d=c.unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
                            //warn!("HAS CTX")
                        }
                        Err(e) => {
                            warn!("NO CTX")
                        }
                    }

*/
                    if (ctw != cw || cth != ch) {
                        self.config.width = cw;
                        self.config.height = ch;
                        let ds=device_state.clone();
                        let dev=ds.read().device.clone();
                        self.surface.configure(&dev.read(), &self.config);
                    }
                }
            }
        }
        self.window.request_redraw();
    }

    pub fn get_window_size(&self) -> PhysicalSize<f32> {
        let scale_factor = 1.0 as f32;
        let is = self.window.inner_size();
        let os = self.window.outer_size();
        PhysicalSize::new(is.width as f32, is.height as f32)
    }
    pub fn get_scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }
    pub fn is_minimized(&self) -> bool {
        match self.window.is_minimized() {
            None => {
                info!("CANT GET is_minimized");
                false
            }
            Some(v) => { v }
        }
    }

    pub fn change_cursor_mode(&mut self) {
        if (self.window_mode == WindowMode::CursorVisible) {
            self.window_mode = WindowMode::CursorInVisible;
            self.hide_cursor();
        } else {
            self.window_mode = WindowMode::CursorVisible;
            self.unhide_cursor();
        }
    }

    fn hide_cursor(&self) {
        let _ = self.window.set_cursor_grab(CursorGrabMode::Confined);
        self.window.set_cursor_visible(false);
    }
    fn unhide_cursor(&self) {
        let _ = self.window.set_cursor_grab(CursorGrabMode::None);
        self.window.set_cursor_visible(true);
    }
    pub fn set_cursor_position(&self, pos: PhysicalPosition<f32>) {
        let _ = self.window.set_cursor_position(pos);
    }
}