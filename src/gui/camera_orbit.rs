use std::f32::consts::PI;
use std::ops::{Sub};
use std::rc::Rc;
use cgmath::{EuclideanSpace, InnerSpace, Point3, Quaternion, Rad, Rotation, Rotation3, Vector3};
use cgmath::num_traits::abs;

use parking_lot::RwLock;
use truck_base::bounding_box::BoundingBox;
use crate::gui::camera_base::{SHIP_FORWARD, SHIP_RIGHT, SHIP_UP};


#[derive(Clone)]
pub struct CameraOrbit {
    pub eye: Rc<RwLock<Point3<f32>>>,
    pub forward: Rc<RwLock<Vector3<f32>>>,
    pub up: Rc<RwLock<Vector3<f32>>>,
    pub right: Rc<RwLock<Vector3<f32>>>,
    pub zoom_factor: f32,
    pub zoom_sensitivity: f32,
    pub dx: f32,
    pub dy: f32,
    yaw: f32,
    pitch: f32,
    mouse_sensitivity_horizontal: f32,
    mouse_sensitivity_vertical: f32,
    pub focus: f32,
}

impl CameraOrbit {
    pub fn new(eye: Rc<RwLock<Point3<f32>>>, head_forward: Rc<RwLock<Vector3<f32>>>, head_up: Rc<RwLock<Vector3<f32>>>, head_right: Rc<RwLock<Vector3<f32>>>) -> Self {
        CameraOrbit {
            eye: eye,
            forward: head_forward,
            up: head_up,
            right: head_right,
            zoom_factor: 50.0,
            zoom_sensitivity:10.0,
            dx: 0.0,
            dy: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            mouse_sensitivity_horizontal: 0.01,
            mouse_sensitivity_vertical: 0.01,
            focus: 700.0,
        }
    }
    pub fn update(&mut self, tot_bbx: &BoundingBox<Point3<f64>>) {
        self.set_zoom_factor((tot_bbx.diagonal().magnitude() / self.zoom_sensitivity as f64) as f32);
    }


    pub fn zoom(&mut self, dx_in: f32, dy_in: f32, d: bool) {
        let k: f32 = if d { 1.0 } else { -1.0 };
        self.pan(dx_in * self.zoom_factor * k, dy_in * self.zoom_factor * k);
        let new_val: Point3<f32> = *self.eye.clone().read() + *self.forward.clone().read() * self.zoom_factor * k;
        self.eye.clone().write().clone_from(&new_val);
    }

    pub fn pan(&mut self, dx_in: f32, dy_in: f32) {
        let eye = self.eye.clone().read().clone();
        let eye_pan_x = eye + self.right.clone().read().clone() * dx_in;
        let eye_pan_y = eye_pan_x + self.up.clone().read().clone() * dy_in;
        self.eye.clone().write().clone_from(&eye_pan_y);
    }

    pub fn update_mouse(&mut self, dx_in: f32, dy_in: f32) {
        self.yaw += dx_in * self.mouse_sensitivity_horizontal;
        self.pitch += dy_in * self.mouse_sensitivity_vertical;
        if abs(self.yaw) > PI * 2.0 { self.yaw = 0.0 }
        if abs(self.pitch) > PI * 2.0 { self.pitch = 0.0 }
        self.rotate();
    }

    pub fn set_zoom_factor(&mut self, bbx_magnitude: f32) {
        if !f32::is_nan(bbx_magnitude) && !f32::is_infinite(bbx_magnitude) {
            self.zoom_factor = bbx_magnitude * 0.1;
        }
    }

    pub fn right_vector(&self) -> Vector3<f32> {
        let forward: Vector3<f32> = self.forward.clone().read().sub(self.eye.clone().read().to_vec());
        let forward_norm: Vector3<f32> = forward.normalize();
        let right: Vector3<f32> = forward_norm.cross(*self.up.clone().read());
        right
    }
    pub fn set_start_pos(&mut self, focus: f32) {
        self.yaw = 0.0;
        self.pitch = 0.0;
        self.focus=focus;
    }


    fn rotate(&mut self) {
        let _up: Vector3<f32> = self.up.clone().read().clone();
        let forward: Vector3<f32> = self.forward.clone().read().clone();
        let _right: Vector3<f32> = self.right.clone().read().clone();
        let eye: Point3<f32> = self.eye.clone().read().clone();

        let new_forward_rot = Quaternion::from_axis_angle(SHIP_UP, Rad(self.yaw)).normalize();
        let new_forward_lr: Vector3<f32> = new_forward_rot.rotate_vector(SHIP_FORWARD);
        let new_right: Vector3<f32> = new_forward_lr.cross(SHIP_UP);

        let new_right_rot = Quaternion::from_axis_angle(new_right, Rad(self.pitch)).normalize();
        let new_forward: Vector3<f32> = new_right_rot.rotate_vector(new_forward_lr);
        let new_up: Vector3<f32> = new_right.cross(new_forward);

        let center: Point3<f32> = eye.clone() + forward * self.focus;
        let new_eye: Point3<f32> = center - new_forward * self.focus;

        self.forward.clone().write().clone_from(&new_forward);
        self.right.clone().write().clone_from(&new_right);
        self.up.clone().write().clone_from(&new_up);
        self.eye.clone().write().clone_from(&new_eye);
    }

    pub fn move_and_look_at(&mut self, new_eye_pos:Point3<f32>, look_at_point:Point3<f32>){
        let dir_raw=look_at_point.sub(new_eye_pos);
        let d=dir_raw.magnitude();
        self.yaw = 0.0;
        self.pitch = 0.0;
        self.forward.clone().write().clone_from(&SHIP_FORWARD);
        self.right.clone().write().clone_from(&SHIP_RIGHT);
        self.up.clone().write().clone_from(&SHIP_UP);
        self.eye.clone().write().clone_from(&new_eye_pos);
        self.focus=d;
    }
}

