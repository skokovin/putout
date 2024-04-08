use std::f32::consts::PI;
use cgmath::{InnerSpace, Matrix4, One, Point3, Quaternion, Rad, Rotation, Rotation3, SquareMatrix, Vector3};
use crate::gui::camera_base::FlyActions;

const SHIP_FORWARD: Vector3<f32> = Vector3::new(1.0, 0.0, 0.0);
const SHIP_RIGHT: Vector3<f32> = Vector3::new(0.0, -1.0, 0.0);
const SHIP_UP: Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);

const LOW_SPEED: f32 = 20.0;
const FAST_SPEED: f32 = 400.0;

const LOW_SPEED_HEAD: f32 = PI / 90.0;
const FAST_SPEED_HEAD: f32 = PI / 90.0;


#[derive(Clone)]
pub struct CameraTouch {
    pub eye: Point3<f32>,
    pub move_up: Vector3<f32>,
    pub move_right: Vector3<f32>,
    pub move_forward: Vector3<f32>,

    pub head_up: Vector3<f32>,
    pub head_right: Vector3<f32>,
    pub head_forward: Vector3<f32>,

    pub quat: Quaternion<f32>,
    pub base_quat: Quaternion<f32>,
    pub dx: f32,
    pub dy: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub velocity_ud: f32,
    pub velocity_fb: f32,
    pub velocity_lr: f32,
    pub velocity_head_ud: f32,
    pub velocity_head_lr: f32,
    pub speed_horizontal: f32,
    pub speed_vertical: f32,
    pub mouse_sensitivity_horizontal: f32,
    pub mouse_sensitivity_vertical: f32,
    pub fly_actions: FlyActions,
    pub view: Matrix4<f32>,
    pub test: f32,

}


impl CameraTouch {
    pub fn new() -> Self {
        CameraTouch {
            eye: Point3::new(0.0, 0.0, 0.0),
            dx: 0.0,
            dy: 0.0,
            move_forward: SHIP_FORWARD.clone(),
            move_right: SHIP_RIGHT.clone(),
            move_up: SHIP_UP.clone(),
            head_forward: SHIP_FORWARD.clone(),
            head_right: SHIP_RIGHT.clone(),
            head_up: SHIP_UP.clone(),
            quat: Quaternion::one(),
            base_quat: Quaternion::one(),
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            velocity_ud: LOW_SPEED,
            velocity_fb: LOW_SPEED,
            velocity_lr: LOW_SPEED,
            velocity_head_ud: LOW_SPEED_HEAD,
            velocity_head_lr: LOW_SPEED_HEAD,
            speed_horizontal: 1.0,
            speed_vertical: 1.0,
            mouse_sensitivity_horizontal: 0.005,
            mouse_sensitivity_vertical: 0.005,
            fly_actions: FlyActions::empty(),
            view: Matrix4::identity(),
            test: 0.0,

        }
    }


    pub fn view(&mut self) -> Matrix4<f32> {
        self.do_transitions();
        Matrix4::look_to_rh(self.eye, self.head_forward, self.head_up)
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

            if self.fly_actions.contains(FlyActions::MOVE_HEAD_L) && !self.fly_actions.contains(FlyActions::MOVE_HEAD_R) {
                self.rotate_head_left();
            }
            if self.fly_actions.contains(FlyActions::MOVE_HEAD_R) && !self.fly_actions.contains(FlyActions::MOVE_HEAD_L) {
                self.rotate_head_right();
            }

            if self.fly_actions.contains(FlyActions::MOVE_FASTER_FB) {
                self.velocity_fb = FAST_SPEED;
            } else {
                self.velocity_fb = LOW_SPEED;
            }
            if self.fly_actions.contains(FlyActions::MOVE_FASTER_LR) {
                self.velocity_lr = FAST_SPEED;
            } else {
                self.velocity_lr = LOW_SPEED;
            }
            if self.fly_actions.contains(FlyActions::MOVE_FASTER_UD) {
                self.velocity_ud = FAST_SPEED;
            } else {
                self.velocity_ud = LOW_SPEED;
            }

            if self.fly_actions.contains(FlyActions::HEAD_FASTER_UD) {
                self.velocity_head_ud = FAST_SPEED;
            } else {
                self.velocity_head_ud = LOW_SPEED;
            }
            if self.fly_actions.contains(FlyActions::HEAD_FASTER_LR) {
                self.velocity_head_lr = FAST_SPEED_HEAD;
            } else {
                self.velocity_head_lr = LOW_SPEED_HEAD;
            }
        }
    }

    fn head_rotation(&mut self, dx: f32, dy: f32) {
        self.yaw += dx;
        self.pitch += dy;
        if self.pitch < -PI / 4.0 { self.pitch = -PI / 4.0 }
        if self.pitch > PI / 4.0 { self.pitch = PI / 4.0 }
        self.rotate();
    }

    fn move_forward(&mut self) {
        self.eye = self.eye + self.move_forward * self.velocity_fb;
    }

    fn move_back(&mut self) {
        self.eye = self.eye - self.move_forward * self.velocity_fb;
    }

    fn move_left(&mut self) {
        self.eye = self.eye - self.move_right * self.velocity_lr;
    }

    fn move_right(&mut self) {
        self.eye = self.eye + self.move_right * self.velocity_lr;
    }

    fn move_up(&mut self) {
        self.eye = self.eye + self.move_up * self.velocity_ud;
    }

    fn move_down(&mut self) {
        self.eye = self.eye - self.move_up * self.velocity_ud;
    }

    fn rotate_head_left(&mut self) {
        self.head_rotation(-self.velocity_head_lr, 0.0)
    }

    fn rotate_head_right(&mut self) {
        self.head_rotation(self.velocity_head_lr, 0.0)
    }

    pub fn move_to_posiions(&mut self, center: Point3<f32>, _eye: Point3<f32>) {
        /*        let offset = {
                    let d = center.distance(eye);
                    if (d < 500.0) { d * 2.0 } else { 15000.0 }
                };*/
        self.eye = Point3::new(center.x, center.y, center.z);
        self.reset_axes();
        self.pitch = 0.0;
        self.yaw = 0.0;
        self.rotate();
    }

    fn reset_axes(&mut self) {
        self.dx = 0.0;
        self.dy = 0.0;
        self.move_forward = SHIP_FORWARD.clone();
        self.move_right = SHIP_RIGHT.clone();
        self.move_up = SHIP_UP.clone();
        self.head_forward = SHIP_FORWARD.clone();
        self.head_right = SHIP_RIGHT.clone();
        self.head_up = SHIP_UP.clone();
    }

    pub fn do_test_action(&mut self) {}

    pub fn add_action(&mut self, action: FlyActions) {
        self.fly_actions = self.fly_actions.union(action);
    }

    pub fn remove_action(&mut self, action: FlyActions) {
        self.fly_actions = self.fly_actions - action;
    }

    pub fn remove_all_heads_actions(&mut self) {
        self.remove_action(FlyActions::HEAD_FASTER_LR);
        self.remove_action(FlyActions::HEAD_FASTER_UD);
        self.remove_action(FlyActions::MOVE_HEAD_L);
        self.remove_action(FlyActions::MOVE_HEAD_R);
        self.remove_action(FlyActions::MOVE_HEAD_D);
        self.remove_action(FlyActions::MOVE_HEAD_U);
    }

    pub fn remove_all_ud_actions(&mut self) {
        self.remove_action(FlyActions::MOVE_FASTER_UD);
        self.remove_action(FlyActions::FLY_UP);
        self.remove_action(FlyActions::FLY_DOWN);
    }

    pub fn remove_all_head_actions(&mut self) {
        self.remove_action(FlyActions::HEAD_FASTER_LR);
        self.remove_action(FlyActions::HEAD_FASTER_UD);
        self.remove_action(FlyActions::MOVE_HEAD_L);
        self.remove_action(FlyActions::MOVE_HEAD_R);
        self.remove_action(FlyActions::MOVE_HEAD_U);
        self.remove_action(FlyActions::MOVE_HEAD_D);
    }

    pub fn remove_all_actions(&mut self) {
        self.fly_actions = FlyActions::empty();
    }

    fn rotate(&mut self) {
        let q_left_right_head: Quaternion<f32> = Quaternion::from_axis_angle(self.head_up, Rad(self.yaw)).normalize();
        self.head_forward = q_left_right_head.rotate_vector(SHIP_FORWARD).normalize();
        self.head_right = self.head_forward.cross(self.head_up).normalize();

        let q_up_down: Quaternion<f32> = Quaternion::from_axis_angle(self.head_right, Rad(self.pitch)).normalize();
        self.head_forward = q_up_down.rotate_vector(self.head_forward).normalize();
        self.head_up = SHIP_UP;


        let q_left_right_move: Quaternion<f32> = Quaternion::from_axis_angle(self.move_up, Rad(self.yaw));
        self.move_forward = q_left_right_move.rotate_vector(SHIP_FORWARD).normalize();
        self.move_right = self.move_forward.cross(self.head_up).normalize();
    }
}
