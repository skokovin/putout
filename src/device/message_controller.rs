use std::collections::HashSet;


use std::rc::Rc;


use cgmath::{Point3};
use cgmath::num_traits::Float;

use log::{info, warn};
use nalgebra::Point4;
use parking_lot::RwLock;
use wgpu::{Device};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceId, ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase};
use winit::keyboard::{KeyCode, PhysicalKey};
use crate::device::window_state::WindowState;
use crate::gui::camera_base::{CameraMode, FlyActions};
use crate::remote::common_state::{COMMANDS, DIMENSIONING, REMOTE_HULL_MESH, SLICER};
use crate::remote::{hull_state, RemoteCommand};
use crate::scene::mesh_loader::read_hull_unpacked_new_format;
use crate::scene::scene_state::SceneState;
use crate::shared::dimension::{Dimension, DimensionMode};
use crate::shared::materials_lib::{EQ_TY_MAX, EQ_TY_MIN, Material, PIPE_TY_MAX, PIPE_TY_MIN, TY_HULL_OTHERS, TY_HULL_OUTERPLATES, TY_HULL_PLATES, TY_HULL_PROFILES};
use crate::shared::mesh_common::MeshVertex;
use crate::shared::shared_buffers::SharedBuffers;
use crate::shared::text_layout::TextLayout;
use crate::shared::Triangle;
#[cfg(target_arch = "wasm32")]
use crate::remote::hull_state::{get_bbx_array, get_index_array, get_types_array, get_vertex_array, on_render_wasm, on_load_to_gpu};

use crate::remote::hull_state::{HIDDEN_HULL, SELECTED_HULL};


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


pub struct MessageController {
    device: Rc<RwLock<Device>>,
    window_state: Rc<RwLock<WindowState>>,
    pub shared_buffers: SharedBuffers,
    pub scene_state: SceneState,
    pub materials: Vec<Material>,
    pub is_materials_dirty: bool,
    contrl: bool,
    shift: bool,
    alt: bool,
    check_counter: i32,

    pub snap_mode: SnapMode,
    pub active_id: u32,
    active_pack_id: u32,
    pub active_point: Point3<f32>,
    pub active_triangle: Triangle,
    pub is_capture_screen_requested: bool,
    pub is_off_screen_ready: bool,
    pub(crate) is_state_dirty: bool,
    pub text_layout: Rc<RwLock<TextLayout>>,
    pub dimension: Dimension,
    pub test_load: i32,
    pub is_wasm_loaded: bool,
    pub is_mouse_btn_active: bool,
}

impl MessageController {
    pub fn new(device: Rc<RwLock<Device>>, window_state: Rc<RwLock<WindowState>>, text_layout: Rc<RwLock<TextLayout>>) -> Self {
        let scene_state = SceneState::new(device.clone());
        let shared_buffers = SharedBuffers::new(device.clone());
        let active_triangle: Triangle = Triangle::new(
            Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        );
        Self {
            device: device,
            window_state: window_state,
            shared_buffers: shared_buffers,
            scene_state: scene_state,
            materials: Material::generate_materials(),
            is_materials_dirty: true,
            contrl: false,
            shift: false,
            alt: false,
            check_counter: 0,

            snap_mode: SnapMode::Disabled,
            active_id: 0,
            active_pack_id: 0,
            active_point: Point3::new(f32::max_value(), f32::max_value(), f32::max_value()),
            active_triangle: active_triangle,
            is_capture_screen_requested: false,
            is_off_screen_ready: false,
            is_state_dirty: false,
            text_layout: text_layout,
            dimension: Dimension::new(),
            test_load: 0,
            is_wasm_loaded: false,
            is_mouse_btn_active: false,
        }
    }


    pub fn on_render(&mut self) {
        #[cfg(target_arch = "wasm32")]
        on_render_wasm();

        // #[cfg(target_arch = "wasm32")]
        self.check_remote_state();
        if (!self.is_wasm_loaded) {
            self.is_wasm_loaded = true;
        }


        self.scene_state.on_render();


        self.text_layout.write().on_render(
            self.snap_mode.clone(),
            self.active_id as i32,
            self.active_point.clone(),
            self.get_mouse_pos(),
            self.dimension.clone(),
            self.scene_state.camera.vp_matrix,
        );
    }
    pub fn on_mouse_move(&mut self, device_id: DeviceId, pos: PhysicalPosition<f64>) {
        let is_dirty_from_mouse = self.scene_state.camera.on_mouse(device_id, pos);
        if is_dirty_from_mouse {
            self.is_state_dirty = is_dirty_from_mouse;
        }
    }

    pub fn on_zoom(&mut self, device_id: DeviceId, delta: MouseScrollDelta, touch_phase: TouchPhase) {
        let is_dirty_from_mouse = self.scene_state.camera.on_zoom(device_id, delta, touch_phase);
       // if is_dirty_from_mouse {
            self.is_state_dirty = true;
       // }
    }
    pub fn on_resize(&mut self, w: u32, h: u32, sf: f64) {
        self.scene_state.camera.resize(w, h);
        self.is_state_dirty = true;
        self.text_layout.write().resize(w, h, sf);
    }
    fn on_keyboard(&mut self, _d: DeviceId, key: KeyEvent, _is_synth: bool) {
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
            PhysicalKey::Code(KeyCode::AltLeft) => {
                match key.state {
                    ElementState::Pressed => {
                        self.alt = true
                    }
                    ElementState::Released => {
                        self.alt = false
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
                        warn!("Start Load Hull by F2{:?}", self.test_load);
                        match self.test_load {
                            0 => {
                                self.scene_state.set_hull_mesh0();
                                self.test_load = self.test_load + 1;
                            }
                            1 => {
                                self.scene_state.set_hull_mesh1();
                                self.test_load = self.test_load + 1;
                            }
                            2 => {
                                self.scene_state.set_hull_mesh2();
                                self.test_load = self.test_load + 1;
                            }
                            3 => {
                                self.scene_state.set_hull_mesh3();
                                self.test_load = self.test_load + 1;
                            }
                            4 => {
                                self.scene_state.set_hull_mesh4();
                                self.test_load = self.test_load + 1;
                            }
                            5 => {
                                self.scene_state.set_hull_mesh5();
                                self.test_load = self.test_load + 1;
                            }
                            6 => {
                                self.scene_state.set_hull_mesh6();
                                self.test_load = self.test_load + 1;
                            }
                            7 => {
                                self.scene_state.set_hull_mesh7();
                                self.test_load = self.test_load + 1;
                            }
                            _ => {}
                        }


                        // info!("FINISH Load Hull by F2{:?}", key);
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
                        self.is_state_dirty = true;
                        info!("FINISH CHange Mode{:?}", key);
                    }
                }
            }

            PhysicalKey::Code(KeyCode::F5) => {
                match key.state {
                    ElementState::Pressed => {}
                    ElementState::Released => {
                        self.set_transparent(0, 1);
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
    fn on_mouse_btn_click(&mut self, _d: DeviceId, state: ElementState, button: MouseButton) {
        match state {
            ElementState::Pressed => {
                self.is_mouse_btn_active = true;
                match button {
                    MouseButton::Left => {
                        match self.alt {
                            true => {
                                self.scene_state.camera.set_frame_pos1();
                            }
                            false => {
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
                                                if self.contrl {
                                                    let is_scene_modified = self.scene_state.screen_oid(ActionType::Hide, self.active_id as i32, self.active_pack_id);
                                                    if is_scene_modified {
                                                        self.is_state_dirty = is_scene_modified;
                                                    }
                                                } else {
                                                    let _is_scene_modified = self.scene_state.screen_oid(ActionType::Select, self.active_id as i32, self.active_pack_id);
                                                }
                                            }
                                            SnapMode::LineDim => {}
                                            SnapMode::NotSet => {}
                                        }
                                    }
                                    CameraMode::TOUCH => {}
                                }
                            }
                        }
                    }
                    MouseButton::Right => {}
                    MouseButton::Middle => {
                        match self.scene_state.camera.mode {
                            CameraMode::FLY => {}
                            CameraMode::ORBIT => {
                                if self.active_id != 0 {
                                    self.scene_state.camera.move_camera_to_pos(self.active_point.clone());
                                    self.is_state_dirty = true;
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
            ElementState::Released => {
                self.is_mouse_btn_active = false;
                match button {
                    MouseButton::Left => {
                        match self.alt {
                            true => {
                                let window_size: PhysicalSize<f32> = self.window_state.read().get_window_size();
                                let sf = self.window_state.read().get_scale_factor() as f32;
                                self.scene_state.camera.set_frame_pos2(window_size, sf);
                                self.is_state_dirty = true;
                            }
                            false => {}
                        }
                    }
                    MouseButton::Right => {}
                    MouseButton::Middle => {}
                    MouseButton::Back => {}
                    MouseButton::Forward => {}
                    MouseButton::Other(_) => {}
                }
            }
        }
    }

    fn check_remote_state(&mut self) {
        match SELECTED_HULL.try_lock() {
            Ok(mut s) => {
                if s.is_dirty {
                    warn!("Start Select HULL");
                    //self.scene_state.select_hull_by_ids(s.values.clone());
                    s.is_dirty = false;
                    s.values.clear();
                    warn!("FINISH Select HULL");
                }
            }
            Err(_) => { warn!("CANT_LOCK") }
        }

        match HIDDEN_HULL.try_lock() {
            Ok(mut s) => {
                if s.is_dirty {
                    warn!("Start HIDE HULL");
                    //self.scene_state.hide_hull_by_ids(s.values.clone());
                    s.is_dirty = false;
                    s.values.clear();
                    warn!("FINISH HIDE HULL");
                }
            }
            Err(_) => { warn!("CANT_LOCK") }
        }

        match SLICER.try_lock() {
            Ok(mut s) => {
                if s.is_dirty {
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
                    None => {
                        if (self.is_state_dirty && !self.is_mouse_btn_active) {
                            self.is_capture_screen_requested = true;
                            self.is_state_dirty = false;
                        }

                    }
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
                                self.scene_state.select_by_ids(ids);

                                self.scene_state.zoom_to(oid);
                            }
                            RemoteCommand::LoadAllToGPU(pack_id) => {
                                match pack_id {
                                    0 => {
                                        self.scene_state.set_hull_mesh0();
                                        #[cfg(target_arch = "wasm32")]
                                        on_load_to_gpu(0);
                                    }
                                    1 => {
                                        self.scene_state.set_hull_mesh1();
                                        #[cfg(target_arch = "wasm32")]
                                        on_load_to_gpu(1);
                                    }
                                    2 => {
                                        self.scene_state.set_hull_mesh2();
                                        #[cfg(target_arch = "wasm32")]
                                        on_load_to_gpu(2);
                                    }
                                    3 => {
                                        self.scene_state.set_hull_mesh3();
                                        #[cfg(target_arch = "wasm32")]
                                        on_load_to_gpu(3);
                                    }
                                    4 => {
                                        self.scene_state.set_hull_mesh4();
                                        #[cfg(target_arch = "wasm32")]
                                        on_load_to_gpu(4);
                                    }
                                    5 => {
                                        self.scene_state.set_hull_mesh5();
                                        #[cfg(target_arch = "wasm32")]
                                        on_load_to_gpu(5);
                                    }
                                    6 => {
                                        self.scene_state.set_hull_mesh6();
                                        #[cfg(target_arch = "wasm32")]
                                        on_load_to_gpu(6);
                                    }
                                    7 => {
                                        self.scene_state.set_hull_mesh7();
                                        #[cfg(target_arch = "wasm32")]
                                        on_load_to_gpu(7);
                                    }
                                    _ => {}
                                }
                            }
                            RemoteCommand::OnMouseMove((id, pos)) => {
                                self.on_mouse_move(id, pos);
                            }

                            RemoteCommand::OnMouseWheel((device_id, delta, touch_phase)) => {
                                self.on_zoom(device_id, delta, touch_phase);
                            }
                            RemoteCommand::OnKeyBoard((d, k, i)) => {
                                self.on_keyboard(d, k, i);
                            }
                            RemoteCommand::OnMouseButton((device_id, state, button)) => {
                                self.scene_state.camera.on_mouse_btn_click(device_id.clone(), state.clone(), button.clone());
                                self.on_mouse_btn_click(device_id.clone(), state.clone(), button.clone());
                            }
                            RemoteCommand::OnOffScreenReady() => {
                                self.is_off_screen_ready = true;
                            }
                            RemoteCommand::SwitchToGameMode() => {
                                if (self.scene_state.camera.mode != CameraMode::FLY) {
                                    self.window_state.clone().write().change_cursor_mode();
                                    self.scene_state.camera.change_mode();
                                    self.is_state_dirty = true;
                                }
                            }
                            RemoteCommand::OnSetTransparentMat((alfa, mode)) => {
                                self.set_transparent(alfa, mode);
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
                if m != self.snap_mode {
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
                if hm.is_dirty {
                    self.scene_state.set_hull_mesh_remote(
                        hm.load_level.clone(),
                        hm.decoded_v.as_slice(),
                        hm.decoded_i.as_slice(),
                        hm.decoded_b.as_slice(),
                        hm.decoded_t.as_slice(),
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
    pub fn set_pack_id(&mut self, active_pack_id: u32) {
        self.active_pack_id = active_pack_id
    }
    pub fn get_pack_id(&self) -> u32 {
        self.active_pack_id
    }
    fn set_transparent(&mut self, alfa: i32, mode: i32) {
        match mode {
            0 => {
                self.materials[TY_HULL_PROFILES as usize].color[3] = alfa as f32 / 100.0;
                self.materials[TY_HULL_PLATES as usize].color[3] = alfa as f32 / 100.0;
                self.materials[TY_HULL_OUTERPLATES as usize].color[3] = alfa as f32 / 100.0;
                self.materials[TY_HULL_OTHERS as usize].color[3] = alfa as f32 / 100.0;
                self.is_materials_dirty = true;
            }
            //PIPE
            1 => {
                self.materials[PIPE_TY_MIN as usize..PIPE_TY_MAX as usize].iter_mut().for_each(|m| {
                    m.color[3] = alfa as f32 / 100.0;
                    warn!("SET PIPE {}", m.color[3]);
                });
                self.is_materials_dirty = true;

            }
            //EQ
            2 => {
                let mut index=EQ_TY_MIN;
                self.materials[EQ_TY_MIN as usize..EQ_TY_MAX as usize].iter_mut().for_each(|m| {
                    if(index!=TY_HULL_PLATES && index!=TY_HULL_OUTERPLATES){
                        m.color[3] = alfa as f32 / 100.0;
                    }

                    warn!("SET EQ {}", m.color[3]);
                    index=index+1;
                });
                self.is_materials_dirty = true;
            }
            _ => {}
        }
    }
}


