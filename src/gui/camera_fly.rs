use std::f32::consts::PI;
use std::rc::Rc;
use cgmath::{InnerSpace, Matrix4, One, Point3, Quaternion, Rad, Rotation, Rotation3, SquareMatrix, Vector3};
use parking_lot::RwLock;
use truck_base::bounding_box::BoundingBox;
use crate::gui::camera_base::{FlyActions, SHIP_FORWARD, SHIP_RIGHT, SHIP_UP};

#[derive(Clone)]
pub struct CameraFly {
    pub eye: Rc<RwLock<Point3<f32>>>,
    pub head_forward: Rc<RwLock<Vector3<f32>>>,
    pub head_up: Rc<RwLock<Vector3<f32>>>,
    pub head_right: Rc<RwLock<Vector3<f32>>>,

    pub move_up: Vector3<f32>,
    pub move_right: Vector3<f32>,
    pub move_forward: Vector3<f32>,

    pub quat: Quaternion<f32>,
    pub base_quat: Quaternion<f32>,
    pub dx: f32,
    pub dy: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub velocity: f32,
    pub speed_horizontal: f32,
    pub speed_vertical: f32,
    pub mouse_sensitivity_horizontal: f32,
    pub mouse_sensitivity_vertical: f32,
    pub fly_actions: FlyActions,
    pub view: Matrix4<f32>,


}


impl CameraFly {
    pub fn new(eye: Rc<RwLock<Point3<f32>>>, head_forward: Rc<RwLock<Vector3<f32>>>, head_up: Rc<RwLock<Vector3<f32>>>, head_right: Rc<RwLock<Vector3<f32>>>) -> Self {
        CameraFly {
            eye: eye,
            head_forward: head_forward,
            head_up: head_up,
            dx: 0.0,
            dy: 0.0,
            move_forward: SHIP_FORWARD.clone(),
            move_right: SHIP_RIGHT.clone(),
            move_up: SHIP_UP.clone(),
            head_right: head_right,
            quat: Quaternion::one(),
            base_quat: Quaternion::one(),
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            velocity: 20.0,
            speed_horizontal: 1.0,
            speed_vertical: 1.0,
            mouse_sensitivity_horizontal: 0.005,
            mouse_sensitivity_vertical: 0.005,
            fly_actions: FlyActions::empty(),
            view: Matrix4::identity(),


        }
    }

    pub fn update(&mut self,tot_bbx: &BoundingBox<Point3<f64>>) {

        self.velocity=(tot_bbx.diagonal().magnitude()/1000.0) as f32;

        self.do_transitions()
    }

    pub fn add_action(&mut self, action: FlyActions) {
        self.fly_actions = self.fly_actions.union(action);
    }

    pub fn update_mouse(&mut self, relative_dx: f32, relative_dy: f32) {
        self.yaw += relative_dx * self.mouse_sensitivity_horizontal;
        self.pitch += relative_dy * self.mouse_sensitivity_vertical;
        if self.pitch < -PI / 4.0 { self.pitch = -PI / 4.0 }
        if self.pitch > PI / 4.0 { self.pitch = PI / 4.0 }
        self.rotate();
    }

    pub fn remove_action(&mut self, action: FlyActions) {
        self.fly_actions = self.fly_actions - action;
    }

    pub fn remove_all_actions(&mut self) {
        self.fly_actions = FlyActions::empty();
    }

    fn do_transitions(&mut self) {
        if !self.fly_actions.is_empty() {
            if self.fly_actions.contains(FlyActions::MOVE_FORWARD) && !self.fly_actions.contains(FlyActions::MOVE_BACKWARD) {
                self.move_forward();
            }
            if self.fly_actions.contains(FlyActions::MOVE_BACKWARD) && !self.fly_actions.contains(FlyActions::MOVE_FORWARD) {
                self.move_back();
            }

            if self.fly_actions.contains(FlyActions::STRAFE_LEFT) && !self.fly_actions.contains(FlyActions::STRAFE_RIGHT) {
                self.move_left();
            }
            if self.fly_actions.contains(FlyActions::STRAFE_RIGHT) && !self.fly_actions.contains(FlyActions::STRAFE_LEFT) {
                self.move_right();
            }

            if self.fly_actions.contains(FlyActions::FLY_DOWN) && !self.fly_actions.contains(FlyActions::FLY_UP) {
                self.move_down();
            }
            if self.fly_actions.contains(FlyActions::FLY_UP) && !self.fly_actions.contains(FlyActions::FLY_DOWN) {
                self.move_up();
            }
            if self.fly_actions.contains(FlyActions::MOVE_FASTER) {
                //self.velocity =  400.0;
            } else {
                //self.velocity = 3.0;
            }
        }
    }

    fn move_forward(&mut self) {
        let new_val: Point3<f32> =*self.eye.clone().read() + self.move_forward * self.velocity;
        self.eye.clone().write().clone_from(&new_val);
    }

    fn move_back(&mut self) {
        let new_val: Point3<f32> =*self.eye.clone().read() - self.move_forward * self.velocity;
        self.eye.clone().write().clone_from(&new_val);
    }

    fn move_left(&mut self) {
        let new_val: Point3<f32> =*self.eye.clone().read() - self.move_right * self.velocity;
        self.eye.clone().write().clone_from(&new_val);
    }

    fn move_right(&mut self) {
        let new_val: Point3<f32> =*self.eye.clone().read() + self.move_right * self.velocity;
        self.eye.clone().write().clone_from(&new_val);
    }

    fn move_up(&mut self) {
        let new_val: Point3<f32> =*self.eye.clone().read() + self.move_up * self.velocity;
        self.eye.clone().write().clone_from(&new_val);
    }

    fn move_down(&mut self) {
        let new_val: Point3<f32> =*self.eye.clone().read() - self.move_up * self.velocity;
        self.eye.clone().write().clone_from(&new_val);
    }

    fn reset_axes(&mut self) {
        self.dx = 0.0;
        self.dy = 0.0;
        self.move_forward = SHIP_FORWARD.clone();
        self.move_right = SHIP_RIGHT.clone();
        self.move_up = SHIP_UP.clone();
        self.head_forward.write().clone_from(&&SHIP_FORWARD.clone());
        self.head_right.write().clone_from(&&SHIP_RIGHT.clone());
        self.head_up.write().clone_from(&&SHIP_UP.clone());
    }

    fn rotate(&mut self) {
        let q_left_right_head: Quaternion<f32> = Quaternion::from_axis_angle(*self.head_up.clone().read(), Rad(self.yaw)).normalize();
        let mut hf: Vector3<f32> = q_left_right_head.rotate_vector(SHIP_FORWARD).normalize();
        let new_head_right=hf.cross(*self.head_up.clone().read()).normalize();
        //self.head_right = hf.cross(*self.head_up.clone().read()).normalize();

        let q_up_down: Quaternion<f32> = Quaternion::from_axis_angle(new_head_right, Rad(self.pitch)).normalize();
        hf = q_up_down.rotate_vector(hf).normalize();
        self.head_up.clone().write().clone_from(&&SHIP_UP.clone());

        let q_left_right_move: Quaternion<f32> = Quaternion::from_axis_angle(self.move_up, Rad(self.yaw));
        self.move_forward = q_left_right_move.rotate_vector(SHIP_FORWARD).normalize();
        self.move_right = self.move_forward.cross(*self.head_up.clone().read()).normalize();

        self.head_forward.clone().write().clone_from(&hf);
        self.head_right.clone().write().clone_from(&new_head_right);

    }

}
