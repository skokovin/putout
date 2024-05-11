use std::ops::Sub;
use std::rc::Rc;
use bitflags::bitflags;
use cgmath::{InnerSpace, Matrix, Matrix4, MetricSpace, perspective, Point3, Rad, SquareMatrix, Vector3, Vector4};
use cgmath::num_traits::{abs, Float, signum};
use log::{info, warn};
use parking_lot::{RwLock};
use truck_base::bounding_box::BoundingBox;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceId, ElementState, MouseButton, MouseScrollDelta, TouchPhase};
use crate::gui::camera_fly::CameraFly;
use crate::gui::camera_orbit::CameraOrbit;
use crate::gui::camera_touch::CameraTouch;
use crate::scene::mesh_loader::Z_FIGHTING_FACTOR;

bitflags!(
    #[repr(transparent)]
    #[derive( Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FlyActions: u32 {
    const MOVE_FORWARD  = 0b00000001;
    const MOVE_BACKWARD = 0b00000010;
    const STRAFE_LEFT   = 0b00000100;
    const STRAFE_RIGHT  = 0b00001000;
    const FLY_UP        = 0b00010000;
    const FLY_DOWN      = 0b00100000;
    const MOVE_FASTER   = 0b01000000;
    const MOVE_FASTER_LR   = 0b0000000010000000;
    const MOVE_FASTER_UD   = 0b0000000100000000;
    const HEAD_FASTER_UD   = 0b0000001000000000;
    const HEAD_FASTER_LR   = 0b0000010000000000;
    const MOVE_FASTER_FB   = 0b0000100000000000;
        const MOVE_HEAD_L  = 0b0001000000000000;
        const MOVE_HEAD_R  = 0b0010000000000000;
        const MOVE_HEAD_U  = 0b0100000000000000;
        const MOVE_HEAD_D  = 0b1000000000000000;
}
);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraMode {
    FLY,
    ORBIT,
    TOUCH,
}

pub const SHIP_FORWARD: Vector3<f32> = Vector3::new(1.0, 0.0, 0.0);
pub const SHIP_RIGHT: Vector3<f32> = Vector3::new(0.0, -1.0, 0.0);
pub const SHIP_UP: Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);

pub struct CameraBase {
    pub eye: Rc<RwLock<Point3<f32>>>,
    pub head_up: Rc<RwLock<Vector3<f32>>>,
    pub head_forward: Rc<RwLock<Vector3<f32>>>,
    pub head_right: Rc<RwLock<Vector3<f32>>>,
    pub index: u32,
    pub name: String,
    pub vp_matrix: Matrix4<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
    pub n_matrix: Matrix4<f32>,
    pub near: f32,
    pub far: f32,
    pub fovy: Rad<f32>,
    pub aspect: f32,
    pub mode: CameraMode,
    pub camera_fly: CameraFly,
    pub camera_orbit: CameraOrbit,
    pub camera_touch: CameraTouch,
    is_mouse_events_disabled: bool,
    mouse_pos: PhysicalPosition<f64>,
    pub mouse_wpos: Point3<f32>,
    pub mouse_wray: Vector3<f32>,
    pub mouse_btn: MouseButton,
    pub screen_w: f32,
    pub screen_h: f32,
    tot_bbx: BoundingBox<Point3<f64>>,
    frame_pos: PhysicalPosition<f64>,
}

impl CameraBase {
    pub fn new(index: u32, fovy: Rad<f32>, aspect: f32, near: f32, far: f32) -> Self {
        let camera_mode = CameraMode::ORBIT;
        let eye = Rc::new(RwLock::new(Point3::new(0.0, 0.0, 0.0)));
        let head_forward = Rc::new(RwLock::new(SHIP_FORWARD.clone()));
        let head_up = Rc::new(RwLock::new(SHIP_UP.clone()));
        let head_right = Rc::new(RwLock::new(SHIP_RIGHT.clone()));

        let camera = CameraBase {
            eye: eye.clone(),
            head_forward: head_forward.clone(),
            head_up: head_up.clone(),
            head_right: head_right.clone(),
            index: index,
            name: "F".to_string(),
            vp_matrix: Matrix4::identity(),
            view: Matrix4::identity(),
            proj: Matrix4::identity(),
            n_matrix: Matrix4::identity(),
            near: near,
            far: far * Z_FIGHTING_FACTOR,
            fovy: fovy,
            aspect: aspect,
            mode: camera_mode,
            camera_fly: CameraFly::new(eye.clone(), head_forward.clone(), head_up.clone(), head_right.clone()),
            camera_orbit: CameraOrbit::new(eye.clone(), head_forward.clone(), head_up.clone(), head_right.clone()),
            camera_touch: CameraTouch::new(),
            is_mouse_events_disabled: false,
            mouse_pos: PhysicalPosition::new(f64::neg_infinity(), f64::neg_infinity()),
            mouse_wpos: Point3::new(0.0, 0.0, 0.0),
            mouse_wray: Vector3::new(1.0, 0.0, 0.0),
            mouse_btn: MouseButton::Other(99),
            screen_w: 1.0,
            screen_h: 1.0,
            tot_bbx: BoundingBox::default(),
            frame_pos: PhysicalPosition::new(f64::neg_infinity(), f64::neg_infinity()),
        };
        camera
    }

    pub fn default() -> Self {
        let cam = CameraBase::new(
            0,
            Rad(std::f32::consts::PI / 3.0),
            1.0,
            0.1,
            200000.0,
        );
        cam
    }

    pub fn get_mvp_buffer(&self) -> &[f32; 16] {
        let view_projection_ref: &[f32; 16] = self.vp_matrix.as_ref();
        view_projection_ref
    }

    pub fn get_mvp_matrix(&self) -> Matrix4<f32> {
        self.vp_matrix.clone()
    }

    pub fn get_norm_buffer(&self) -> &[f32; 16] {
        let view_projection_ref: &[f32; 16] = self.n_matrix.as_ref();
        view_projection_ref
    }
    pub fn get_forward_dir_buffer(&self) -> [f32; 3] {
        let v = self.head_forward.read();
        let out: [f32; 3] = [v.x, v.y, v.z];
        out
    }

    pub fn update(&mut self, tot_bbx: BoundingBox<Point3<f64>>) {
        let diag = tot_bbx.diagonal().magnitude();
        self.tot_bbx = tot_bbx;
        match self.mode {
            CameraMode::FLY => { self.camera_fly.update(&self.tot_bbx) }
            CameraMode::ORBIT => { self.camera_orbit.update(&self.tot_bbx) }
            CameraMode::TOUCH => {}
        };
        self.view = Matrix4::look_to_rh(*self.eye.clone().read(), *self.head_forward.clone().read(), *self.head_up.clone().read());
        self.proj = perspective(self.fovy, self.aspect, self.near, self.far);
        // let scale=Matrix4::from_scale(5.0 as f32);
        self.vp_matrix = self.proj * self.view;

        self.n_matrix = self.view.transpose();
        if diag.is_normal() {
            self.far = diag as f32;
        }
        self.mouse_screen_to_world();
    }


    fn mouse_screen_to_world(&mut self) {
        let x = (2.0 as f32 * self.mouse_pos.x as f32) / self.screen_w - 1.0;
        let y = 1.0 as f32 - (2.0 as f32 * self.mouse_pos.y as f32) / self.screen_h;
        let ray_clip = Vector4::new(x, y, -1.0, 1.0);
        let _ray_eye = (self.proj.invert().unwrap()) * ray_clip;
        let ray_eye = Vector4::new(_ray_eye.x, _ray_eye.y, -1.0, 0.0);
        let inv_ray_wor = (self.view.invert().unwrap()) * ray_eye;
        let _ray_wor = Vector3::new(inv_ray_wor.x, inv_ray_wor.y, inv_ray_wor.z);
        let ray_wor = _ray_wor.normalize();
        let mut pos = (self.vp_matrix.clone().invert().unwrap()) * ray_clip;
        pos.w = 1.0 / pos.w;
        pos.x *= pos.w;
        pos.y *= pos.w;
        pos.z *= pos.w;
        let p = Point3::new(pos.x, pos.y, pos.z);
        self.mouse_wpos = p;
        self.mouse_wray = ray_wor.clone();
    }

    fn mouse_screen_to_world_by_2d(&self, _x: f32, _y: f32) -> Point3<f32> {
        let x = (2.0 as f32 * _x as f32) / self.screen_w - 1.0;
        let y = 1.0 as f32 - (2.0 as f32 * _y as f32) / self.screen_h;
        let ray_clip = Vector4::new(x, y, -1.0, 1.0);
        //let _ray_eye = (self.proj.invert().unwrap()) * ray_clip;
        //let ray_eye = Vector4::new(_ray_eye.x, _ray_eye.y, -1.0, 0.0);
        //let inv_ray_wor = (self.view.invert().unwrap()) * ray_eye;
        //let _ray_wor = Vector3::new(inv_ray_wor.x, inv_ray_wor.y, inv_ray_wor.z);
        //let ray_wor = _ray_wor.normalize();
        let mut pos = (self.vp_matrix.clone().invert().unwrap()) * ray_clip;
        pos.w = 1.0 / pos.w;
        pos.x *= pos.w;
        pos.y *= pos.w;
        pos.z *= pos.w;
        let p: Point3<f32> = Point3::new(pos.x, pos.y, pos.z);
        p
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w > 0 && h > 0 {
            self.near = 0.1;
            self.aspect = w as f32 / h as f32;
            self.screen_w = w as f32;
            self.screen_h = h as f32;
        }
    }

    pub fn position(&self) -> Point3<f32> {
        *self.eye.clone().read()
    }

    pub fn change_mode(&mut self) {
        match self.mode {
            CameraMode::FLY => { self.mode = CameraMode::ORBIT }
            CameraMode::ORBIT => { self.mode = CameraMode::FLY; }
            CameraMode::TOUCH => {  }
        }
    }

    //FLY AREA

    pub fn add_action(&mut self, action: FlyActions) {
        match self.mode {
            CameraMode::FLY => { self.camera_fly.add_action(action); }
            CameraMode::ORBIT => {}
            CameraMode::TOUCH => { self.camera_touch.add_action(action); }
        }
    }

    pub fn move_and_look_at(&mut self, new_eye_pos: Point3<f32>, look_at_point: Point3<f32>) {
        match self.mode {
            CameraMode::FLY => {}
            CameraMode::ORBIT => {
                self.camera_orbit.move_and_look_at(new_eye_pos, look_at_point);
            }
            CameraMode::TOUCH => {}
        }
    }

    pub fn remove_action(&mut self, action: FlyActions) {
        match self.mode {
            CameraMode::FLY => { self.camera_fly.remove_action(action); }
            CameraMode::ORBIT => {}
            CameraMode::TOUCH => { self.camera_touch.remove_action(action); }
        }
    }

    pub fn relese_mouse(&mut self) {
        self.mouse_pos = PhysicalPosition::new(f64::neg_infinity(), f64::neg_infinity());
    }

    pub fn on_mouse(&mut self, _device_id: DeviceId, pos: PhysicalPosition<f64>) -> bool {
        let mut set_is_dirty = false;
        if self.mouse_pos.x == f64::neg_infinity() {
            self.mouse_pos = pos.clone();
            set_is_dirty
        } else {
            let dx = self.mouse_pos.x - pos.x;
            let dy = self.mouse_pos.y - pos.y;
            match self.mode {
                CameraMode::FLY => {
         /*           if abs(dx) < 100.0 && abs(dy) < 100.0 {
                        self.camera_fly.update_mouse(dx as f32, dy as f32);
                    }*/
                }
                CameraMode::ORBIT => {
                   /* match self.mouse_btn {
                        MouseButton::Left => {}
                        MouseButton::Right => {
                            if abs(dx) < 100.0 && abs(dy) < 100.0 {
                                self.camera_orbit.update_mouse(dx as f32, dy as f32);
                                set_is_dirty = true;
                            }
                        }
                        MouseButton::Middle => {
                            if abs(dx) < 100.0 && abs(dy) < 100.0 {
                                self.camera_orbit.pan(dx as f32, dy as f32);
                                set_is_dirty = true;
                            }
                        }
                        MouseButton::Back => {}
                        MouseButton::Forward => {}
                        MouseButton::Other(_) => {}
                    }*/
                }
                CameraMode::TOUCH => {}
            };
            self.mouse_pos = pos.clone();
            //warn!("{:?} {} {}", self.mouse_pos, dx,dy);
            set_is_dirty
        }
    }

    pub fn on_mouse_dx_dy(&mut self, _device_id: DeviceId, dx: f64, dy: f64) {
        match self.mode {
            CameraMode::FLY => {
                self.camera_fly.update_mouse(-dx as f32, dy as f32);
            }
            CameraMode::ORBIT => {
                match self.mouse_btn {
                    MouseButton::Left => {}
                    MouseButton::Right => {
                        if abs(dx) < 100.0 && abs(dy) < 100.0 {
                            self.camera_orbit.update_mouse(-dx as f32, -dy as f32);
                            //set_is_dirty = true;
                        }
                    }
                    MouseButton::Middle => {
                        if abs(dx) < 100.0 && abs(dy) < 100.0 {
                            self.camera_orbit.pan(-dx as f32, dy as f32);
                            //set_is_dirty = true;
                        }
                    }
                    MouseButton::Back => {}
                    MouseButton::Forward => {}
                    MouseButton::Other(_) => {}
                }
            }
            CameraMode::TOUCH => {}
        };
    }
    pub fn on_mouse_btn_click(&mut self, _device_id: DeviceId, state: ElementState, btn: MouseButton) {
        match state {
            ElementState::Pressed => {
                self.mouse_btn = btn;
                match self.mouse_btn {
                    MouseButton::Left => {}
                    MouseButton::Right => {}
                    MouseButton::Middle => {
                        match self.mode {
                            CameraMode::FLY => {}
                            CameraMode::ORBIT => {}
                            CameraMode::TOUCH => {}
                        }
                    }
                    MouseButton::Back => {}
                    MouseButton::Forward => {}
                    MouseButton::Other(_) => {}
                }
            }
            ElementState::Released => { self.mouse_btn = MouseButton::Other(99) }
        }
    }
    pub fn on_zoom(&mut self, _device_id: DeviceId, delta: MouseScrollDelta, _touch_phase: TouchPhase) -> bool {
        let mut set_is_dirty = false;
        match self.mode {
            CameraMode::FLY => {}
            CameraMode::ORBIT => {
                let dx = (self.mouse_pos.x as f32 - self.screen_w / 2.) / (self.screen_w / 2.);
                let dy = -(self.mouse_pos.y as f32 - self.screen_h / 2.) / (self.screen_h / 2.);
                match delta {
                    MouseScrollDelta::LineDelta(_horiz, vert) => {
                        self.camera_orbit.zoom(dx, dy, signum(vert).is_sign_negative());
                    }
                    MouseScrollDelta::PixelDelta(d) => {
                        self.camera_orbit.zoom(dx, dy, signum(d.y).is_sign_negative());
                    }
                }
                set_is_dirty = true
            }
            CameraMode::TOUCH => {}
        };
        set_is_dirty
    }
    pub fn move_camera_to_pos(&mut self, new_center_point: Point3<f32>) {
        let e = self.eye.clone().read().clone();
        let v = new_center_point.sub(e);
        let dist = v.x * self.head_forward.read().x + v.y * self.head_forward.read().y + v.z * self.head_forward.read().z;
        let projected_point = new_center_point.sub(dist * self.head_forward.clone().read().clone());
        let new_focus = projected_point.sub(new_center_point).magnitude();
        let en: Point3<f32> = new_center_point + self.head_forward.clone().read().clone() * -new_focus;
        self.eye.clone().write().clone_from(&en);
        self.camera_orbit.focus = new_focus;
    }
    pub fn move_camera_to_startpos(&mut self) {
        let focus = self.tot_bbx.diagonal().magnitude() / 2.0;
        let p: Point3<f32> = Point3::new(0.0, 0.0, 0.0);
        self.eye.clone().write().clone_from(&p);
        self.head_forward.clone().write().clone_from(&SHIP_FORWARD);
        self.head_up.clone().write().clone_from(&SHIP_UP);
        self.head_right.clone().write().clone_from(&SHIP_RIGHT);
        self.camera_orbit.set_start_pos(focus as f32);
    }
    pub fn get_mouse_pos(&self, _scale_factor: f64) -> PhysicalPosition<f64> {
        self.mouse_pos.clone()
    }

    pub fn set_frame_pos1(&mut self) {
        self.frame_pos = self.mouse_pos.clone();
    }
    pub fn set_frame_pos2(&mut self, window_size: PhysicalSize<f32>, scale_factor: f32) {
        match self.mode {
            CameraMode::FLY => {}
            CameraMode::ORBIT => {
                let x = self.screen_w.clone();
                let y = self.screen_h.clone();

                let p0: Point3<f32> = self.mouse_screen_to_world_by_2d(0.0, 0.0);
                let p1: Point3<f32> = self.mouse_screen_to_world_by_2d(x, y);

                let d: f32 = p0.distance(p1);
                //println!("DIST {:?}  {:?}  {:?}  ",p0,p1,d);

                self.camera_orbit.zoom_by_frame(self.frame_pos.clone(), self.mouse_pos.clone(), window_size, scale_factor, d);
            }
            CameraMode::TOUCH => {}
        }
    }
}

