use std::cell::{RefCell, RefMut};
use std::collections::HashSet;
use std::future::Future;
use std::ops::Sub;
use std::rc::Rc;
use std::sync::{LockResult, TryLockResult};
use tokio::sync::mpsc::{Receiver, Sender};
use cgmath::{InnerSpace, Point3, Vector3, Vector4};
use itertools::Itertools;
use log::{info, Level, warn};
use nalgebra::Point4;
use parking_lot::RwLock;

use wgpu::{Device, Queue};
use wgpu::PolygonMode::Point;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase};
use winit::keyboard;
use winit::keyboard::{Key, KeyCode, NamedKey, NativeKey, PhysicalKey, SmolStr};
use crate::device::window_state::WindowState;
use crate::gui::camera_base::{CameraMode, FlyActions};
use crate::remote;
use crate::remote::common_state::{COMMANDS, debug_move_to, DIMENSIONING, REMOTE_HULL_MESH, SLICER};
use crate::remote::{hull_state, RemoteCommand};
use crate::remote::hull_state::{HIDDEN_HULL, SELECTED_HULL};
use crate::scene::{mesh_loader, RawMesh};
use crate::scene::scene_state::SceneState;
use crate::shared::dimension::{Dimension, DimensionMode};
use crate::shared::materials_lib::Material;
use crate::shared::mesh_common::{MeshDrawIndexedIndirect, MeshVertex};

use crate::shared::mesh_pipeline::{MeshPipeLine};
use crate::shared::shared_buffers::SharedBuffers;
use crate::shared::text_layout::TextLayout;
use crate::shared::Triangle;

const DELAY: i32 = 50;

#[derive(PartialEq, Clone)]
pub enum SnapMode {
    Vertex = 0,
    Edge = 1,
    Face = 2,
    Disabled = 3,
    LineDim = 4,
    NotSet = 5,
}

#[derive(PartialEq)]
pub enum ActionType {
    Select,
    Hide,
    Evaluate,
}

#[derive(PartialEq)]
pub enum SMEvent {
    LoadHull(i32),
    KeyBoardEvent((DeviceId, KeyEvent, bool)),
    MouseButtonEvent((DeviceId, ElementState, MouseButton)),
    SelectedPixels(i32),
}

pub struct MessageController {
    sender: Sender<SMEvent>,
    receiver: Receiver<SMEvent>,
    device: Rc<RwLock<Device>>,
    window_state: Rc<RwLock<WindowState>>,
    pub shared_buffers: SharedBuffers,
    pub scene_state: SceneState,
    pub materials: Vec<Material>,
    pub is_materials_dirty: bool,
    contrl: bool,
    shift: bool,
    check_counter: i32,
    pub snap_point: Point4<f32>,
    pub snap_mode: SnapMode,
    pub active_id: i32,
    pub active_point: Point3<f32>,
    pub active_triangle: Triangle,
    pub is_capture_screen_requested: bool,
    is_state_dirty: bool,
    is_state_dirty_delay_counter: i32,
    pub text_layout: Rc<RwLock<TextLayout>>,
    pub dimension: Dimension,

}

impl MessageController {
    pub fn new(device: Rc<RwLock<Device>>, window_state: Rc<RwLock<WindowState>>, text_layout: Rc<RwLock<TextLayout>>) -> Self {
        let (tx, rx): (Sender<SMEvent>, Receiver<SMEvent>) = tokio::sync::mpsc::channel(64);
        let scene_state = SceneState::new(device.clone());
        let shared_buffers = SharedBuffers::new(device.clone());
        let active_triangle: Triangle = Triangle::new(
            Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        );
        Self {
            sender: tx,
            receiver: rx,
            device: device,
            window_state: window_state,
            shared_buffers: shared_buffers,
            scene_state: scene_state,
            materials: Material::generate_materials(),
            is_materials_dirty: true,
            contrl: false,
            shift: false,
            check_counter: 0,
            snap_point: Point4::new(480.0, 28.0, -90.0, 1.0),
            snap_mode: SnapMode::Disabled,
            active_id: 0,
            active_point: Point3::new(0.0, 0.0, 0.0),
            active_triangle: active_triangle,
            is_capture_screen_requested: false,
            is_state_dirty: false,
            is_state_dirty_delay_counter: DELAY,
            text_layout: text_layout,
            dimension: Dimension::new(),
        }
    }
    pub fn get_sender(&self) -> Sender<SMEvent> {
        self.sender.clone()
    }
    pub fn on_render(&mut self) {
        self.check_remote_state();
        self.scene_state.on_render();
        match self.receiver.try_recv() {
            Ok(event) => {
                match event {
                    SMEvent::LoadHull(i) => {
                        warn!("LOAD HULL REQUEST{:?}", i);
                    }
                    SMEvent::KeyBoardEvent((d, k, i)) => { self.on_keyboard(d, k, i); }
                    SMEvent::MouseButtonEvent((device_id, state, button)) => {
                        self.scene_state.camera.on_mouse_btn_click(device_id.clone(), state.clone(), button.clone());
                        self.on_mouse_btn_click(device_id.clone(), state.clone(), button.clone());
                    }
                    SMEvent::SelectedPixels(id) => {
                        warn!("PIXELS{:?}", id);
                    }
                }
            }
            Err(e) => {}
        }
        self.check_delay();
        self.text_layout.write().on_render(
            self.snap_mode.clone(),
            self.active_id,
            self.active_point.clone(),
            self.get_mouse_pos(),
            self.dimension.clone(),
            self.scene_state.camera.vp_matrix,
        );
    }
    pub fn on_mouse_move(&mut self, device_id: DeviceId, pos: PhysicalPosition<f64>) {
        let is_dirty_from_mouse = self.scene_state.camera.on_mouse(device_id, pos);
        if (is_dirty_from_mouse) {
            self.is_state_dirty = is_dirty_from_mouse;
        }
    }
    pub fn on_zoom(&mut self, device_id: DeviceId, delta: MouseScrollDelta, touch_phase: TouchPhase) {
        let is_dirty_from_mouse = self.scene_state.camera.on_zoom(device_id, delta, touch_phase);
        if (is_dirty_from_mouse) {
            self.is_state_dirty = is_dirty_from_mouse;
        }
    }
    pub fn on_resize(&mut self, w: u32, h: u32, sf: f64) {
        self.scene_state.camera.resize(w, h);
        self.is_state_dirty = true;
        self.text_layout.write().resize(w, h, sf);
    }
    fn on_keyboard(&mut self, d: DeviceId, key: KeyEvent, is_synth: bool) {
        match key.physical_key {
            PhysicalKey::Code(KeyCode::F6) => {
                match key.state {
                    ElementState::Pressed => {}
                    ElementState::Released => {
                        warn!("Start Slicer");
                        //unsafe { debug_move_to(); }
                        warn!("FINISH Slicer");
                    }
                }
            }

            PhysicalKey::Code(KeyCode::ControlLeft) => {
                match key.state {
                    ElementState::Pressed => {
                        self.contrl = true
                    }
                    ElementState::Released => {
                        self.contrl = false
                    }
                }
            }
            PhysicalKey::Code(KeyCode::ShiftLeft) => {
                match key.state {
                    ElementState::Pressed => {
                        self.shift = true
                    }
                    ElementState::Released => {
                        self.shift = false
                    }
                }
            }

            PhysicalKey::Code(KeyCode::F9) => {
                match key.state {
                    ElementState::Pressed => {}
                    ElementState::Released => {
                        match DIMENSIONING.lock() {
                            Ok(mut m) => {
                                match self.snap_mode {
                                    SnapMode::Vertex => { *m = SnapMode::Edge; }
                                    SnapMode::Edge => { *m = SnapMode::Face; }
                                    SnapMode::Face => { *m = SnapMode::Disabled; }
                                    SnapMode::Disabled => { *m = SnapMode::LineDim; }
                                    SnapMode::LineDim => { *m = SnapMode::Vertex; }
                                    SnapMode::NotSet => { *m = SnapMode::Vertex; }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }
            }

            PhysicalKey::Code(KeyCode::F2) => {
                match key.state {
                    ElementState::Pressed => {}
                    ElementState::Released => {
                        info!("Start Load Hull by F2{:?}", key);
                        self.scene_state.set_hull_mesh();
                        info!("FINISH Load Hull by F2{:?}", key);
                    }
                }
            }
            PhysicalKey::Code(KeyCode::F3) => {
                match key.state {
                    ElementState::Pressed => {}
                    ElementState::Released => {
                        info!("CHange Mode{:?}", key);
                        self.window_state.clone().write().change_cursor_mode();
                        self.scene_state.camera.change_mode();
                        info!("FINISH CHange Mode{:?}", key);
                    }
                }
            }
            PhysicalKey::Code(KeyCode::F4) => {
                match key.state {
                    ElementState::Pressed => {}
                    ElementState::Released => {
                        let mut v: HashSet<i32> = HashSet::new();
                        let selmeshes = self.scene_state.hull_mesh.iter().take(2500);
                        selmeshes.for_each(|m| {
                            v.insert(m.0.clone());
                        });

                        match hull_state::SELECTED_HULL.try_lock() {
                            Ok(mut s) => {
                                s.values = v;
                                s.is_dirty = true;
                            }
                            Err(_) => { warn!("CANT_LOCK") }
                        }
                    }
                }
            }

            PhysicalKey::Code(KeyCode::F5) => {
                match key.state {
                    ElementState::Pressed => {}
                    ElementState::Released => {

                    }
                }
            }

            PhysicalKey::Code(KeyCode::KeyW) => {
                match key.state {
                    ElementState::Pressed => {
                        self.scene_state.camera.add_action(FlyActions::MOVE_FORWARD);
                    }
                    ElementState::Released => {
                        self.scene_state.camera.remove_action(FlyActions::MOVE_FORWARD);
                    }
                }
            }
            PhysicalKey::Code(KeyCode::KeyS) => {
                match key.state {
                    ElementState::Pressed => {
                        self.scene_state.camera.add_action(FlyActions::MOVE_BACKWARD);
                    }
                    ElementState::Released => {
                        self.scene_state.camera.remove_action(FlyActions::MOVE_BACKWARD);
                    }
                }
            }
            PhysicalKey::Code(KeyCode::KeyA) => {
                match key.state {
                    ElementState::Pressed => {
                        self.scene_state.camera.add_action(FlyActions::STRAFE_LEFT);
                    }
                    ElementState::Released => {
                        self.scene_state.camera.remove_action(FlyActions::STRAFE_LEFT);
                    }
                }
            }
            PhysicalKey::Code(KeyCode::KeyD) => {
                match key.state {
                    ElementState::Pressed => {
                        self.scene_state.camera.add_action(FlyActions::STRAFE_RIGHT);
                    }
                    ElementState::Released => {
                        self.scene_state.camera.remove_action(FlyActions::STRAFE_RIGHT);
                    }
                }
            }
            PhysicalKey::Code(KeyCode::KeyE) => {
                match key.state {
                    ElementState::Pressed => {
                        self.scene_state.camera.add_action(FlyActions::FLY_UP);
                    }
                    ElementState::Released => {
                        self.scene_state.camera.remove_action(FlyActions::FLY_UP);
                    }
                }
            }
            PhysicalKey::Code(KeyCode::KeyC) => {
                match key.state {
                    ElementState::Pressed => {
                        self.scene_state.camera.add_action(FlyActions::FLY_DOWN);
                    }
                    ElementState::Released => {
                        self.scene_state.camera.remove_action(FlyActions::FLY_DOWN);
                    }
                }
            }

            PhysicalKey::Code(KeyCode::KeyP) => {
                match key.state {
                    ElementState::Pressed => {
                        //self.materials[0].increase_ambient();
                        //info!("AMBIENT {}",self.materials[0].ambient_intensity);
                    }
                    ElementState::Released => {}
                }
            }
            PhysicalKey::Code(KeyCode::KeyO) => {
                match key.state {
                    ElementState::Pressed => {
                        //self.materials[0].decrease_ambient();
                        //("AMBIENT {}",self.materials[0].ambient_intensity);
                    }
                    ElementState::Released => {}
                }
            }
            PhysicalKey::Code(KeyCode::KeyI) => {
                match key.state {
                    ElementState::Pressed => {
                        //self.materials[0].increase_ambient();
                        //info!("AMBIENT {}",self.materials[0].ambient_intensity);
                    }
                    ElementState::Released => {}
                }
            }
            PhysicalKey::Code(KeyCode::KeyU) => {
                match key.state {
                    ElementState::Pressed => {
                        //self.materials[0].decrease_ambient();
                        //info!("AMBIENT {}",self.materials[0].ambient_intensity);
                    }
                    ElementState::Released => {}
                }
            }

            PhysicalKey::Unidentified(_) => {}
            _ => (),
        }
    }
    fn on_mouse_btn_click(&mut self, d: DeviceId, state: ElementState, button: MouseButton) {
        match state {
            ElementState::Pressed => {
                match button {
                    MouseButton::Left => {
                        //warn!("POINT ID IS  {:?}",self.active_point);
                        self.snap_point = Point4::new(self.active_point.x, self.active_point.y, self.active_point.z, 1.0);
                        match self.scene_state.camera.mode {
                            CameraMode::FLY => {}
                            CameraMode::ORBIT => {
                                match self.snap_mode {
                                    SnapMode::Vertex => {
                                        self.dimension.set_point(self.active_point, DimensionMode::Line);
                                    }
                                    SnapMode::Edge => {}
                                    SnapMode::Face => {}
                                    SnapMode::Disabled => {
                                        if (self.contrl) {
                                            let is_scene_modified = self.scene_state.screen_oid(ActionType::Hide, self.active_id);
                                            if (is_scene_modified) {
                                                self.is_state_dirty = is_scene_modified;
                                            }
                                        } else {
                                            let is_scene_modified = self.scene_state.screen_oid(ActionType::Select, self.active_id);
                                        }
                                    }
                                    SnapMode::LineDim => {}
                                    SnapMode::NotSet => {}
                                }
                            }
                            CameraMode::TOUCH => {}
                        }
                    }
                    MouseButton::Right => {}
                    MouseButton::Middle => {
                        match self.scene_state.camera.mode {
                            CameraMode::FLY => {}
                            CameraMode::ORBIT => {
                                if (self.active_id != 0) {
                                    self.scene_state.camera.move_camera_to_pos(self.active_point.clone());
                                }
                            }
                            CameraMode::TOUCH => {}
                        }
                    }
                    MouseButton::Back => {}
                    MouseButton::Forward => {}
                    MouseButton::Other(_) => {}
                }
            }
            ElementState::Released => {}
        }
    }
    fn check_remote_state(&mut self) {
        match SELECTED_HULL.try_lock() {
            Ok(mut s) => {
                if (s.is_dirty) {
                    warn!("Start Select HULL");
                    self.scene_state.select_hull_by_ids(s.values.clone());
                    s.is_dirty = false;
                    s.values.clear();
                    warn!("FINISH Select HULL");
                }
            }
            Err(_) => { warn!("CANT_LOCK") }
        }

        match HIDDEN_HULL.try_lock() {
            Ok(mut s) => {
                if (s.is_dirty) {
                    warn!("Start HIDE HULL");
                    self.scene_state.hide_hull_by_ids(s.values.clone());
                    s.is_dirty = false;
                    s.values.clear();
                    warn!("FINISH HIDE HULL");
                }
            }
            Err(_) => { warn!("CANT_LOCK") }
        }

        match SLICER.try_lock() {
            Ok(mut s) => {
                if (s.is_dirty) {
                    self.scene_state.slicer.set_slicer(
                        s.values[0],
                        s.values[1],
                        s.values[2],
                        s.values[3],
                        s.values[4],
                        s.values[5]);
                    s.is_dirty = false;
                    s.values.clear();
                }
            }
            Err(_) => { warn!("CANT_LOCK") }
        }

        match COMMANDS.try_lock() {
            Ok(mut s) => {
                match s.get_first() {
                    None => {}
                    Some(command) => {
                        match command {
                            RemoteCommand::MoveCameraToStartPos => {
                                self.scene_state.camera.move_camera_to_startpos();
                            }
                            RemoteCommand::MoveCameraToOID(oid) => {
                                warn!("GOT MoveCameraToOID");
                                //debugonly!!!
                                let mut ids: HashSet<i32> = HashSet::new();
                                ids.insert(oid.clone());
                                self.scene_state.select_hull_by_ids(ids);

                                self.scene_state.zoom_to(oid);
                            }
                        }
                    }
                }
            }
            Err(_) => { warn!("CANT_LOCK") }
        }

        match DIMENSIONING.try_lock() {
            Ok(mode) => {
                let m = mode.clone();
                if (m != self.snap_mode) {
                    match m {
                        SnapMode::Vertex => { self.snap_mode = SnapMode::Vertex; }
                        SnapMode::Edge => { self.snap_mode = SnapMode::Edge; }
                        SnapMode::Face => { self.snap_mode = SnapMode::Face; }
                        SnapMode::Disabled => {
                            self.snap_mode = SnapMode::Disabled;
                            self.dimension.clear();
                            self.text_layout.write().clear_dimension_value();
                        }
                        SnapMode::LineDim => { self.snap_mode = SnapMode::LineDim; }
                        SnapMode::NotSet => { self.snap_mode = SnapMode::NotSet; }
                    }
                }
            }
            Err(_) => { warn!("CANT_LOCK") }
        }

        match REMOTE_HULL_MESH.try_lock() {
            Ok(mut hm) => {
                if(hm.is_dirty){
                    self.scene_state.set_hull_mesh_remote(
                        hm.decoded_v.clone(),
                        hm.decoded_i.clone(),
                        hm.decoded_b.clone(),
                        hm.decoded_t.clone(),
                    );
                    hm.clean();
                }
            }
            Err(_) => {}
        }
    }
    pub fn reset_material_dirty(&mut self) {
        self.is_materials_dirty = false;
    }

    pub fn get_mouse_pos(&self) -> PhysicalPosition<f64> {
        self.scene_state.camera.get_mouse_pos(self.window_state.read().get_scale_factor())
    }
    fn check_delay(&mut self) {
        if (self.is_state_dirty) {
            self.is_state_dirty = false;
            self.is_state_dirty_delay_counter = DELAY;
            self.is_state_dirty_delay_counter = self.is_state_dirty_delay_counter - 1;
        }

        if (self.is_state_dirty_delay_counter != DELAY) {
            self.is_state_dirty_delay_counter = self.is_state_dirty_delay_counter - 1;
        }

        if (!self.is_state_dirty && self.is_state_dirty_delay_counter < 0) {
            self.is_state_dirty_delay_counter = DELAY;
            self.is_capture_screen_requested = true;
            println!("CAPTURE REQURSTED");
        }
    }
}

