use std::f32::consts::PI;
use std::ops::{Mul, Sub};
use cgmath::{Basis3, InnerSpace, Matrix4, Point3, Rad, Rotation, Rotation3, Transform, Vector3};
use serde::{Deserialize, Serialize};

use truck_base::bounding_box::BoundingBox;
use crate::scene::RawMesh;
use crate::shared::{ANGLE_SUBDIVISIONS, CABLE_EDGE_COLOR, CABLE_EDGE_RADIUS, CABLE_NODE_COLOR, PIPE_PRIME_TYPE, Triangle};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PipePrimitive {
    pub id: i32,
    pub p_a: Point3<f32>,
    pub p_b: Point3<f32>,
    pub radius: f32,
    pub mat_idx: i32,
}

impl PipePrimitive {
    pub fn new(id: i32, p_a: Point3<f32>, p_b: Point3<f32>, radius: f32,  mat_idx: i32,) -> Self {
        Self {
            id: id,
            p_a: p_a,
            p_b: p_b,
            radius: radius,
            mat_idx: mat_idx,
        }
    }

    pub fn triangulate(&self) -> RawMesh {
        let mut triangles: Vec<Triangle> = vec![];
        let mut vrtxs: Vec<f32> = vec![];
        let mut indxs: Vec<i32> = vec![];
        let step_angle = PI * 2.0 / ANGLE_SUBDIVISIONS as f32;
        let step_half_angle = PI / ANGLE_SUBDIVISIONS as f32;

        let v_orig: Vector3<f32> =self.p_b.sub(self.p_a);
        let v: Vector3<f32> = v_orig.normalize();
        let mut radius_dir1: Vector3<f32> = PipePrimitive::generate_perpendicular(v);

        let translation0 = Matrix4::from_translation(radius_dir1.mul(CABLE_EDGE_RADIUS));
        let mut p0: Point3<f32> = translation0.transform_point(self.p_a);
        let mut p1: Point3<f32> = translation0.transform_point(self.p_b);

        let bbx = {
            let translation2 = Matrix4::from_translation(radius_dir1.mul(-CABLE_EDGE_RADIUS));
            let max_p: Point3<f32> = translation0.transform_point(self.p_a);
            let min_p: Point3<f32> = translation2.transform_point(self.p_b);
            let bbx_min: Point3<f64> = Point3::new(min_p.x as f64, min_p.y as f64, min_p.z as f64);
            let bbx_max: Point3<f64> = Point3::new(max_p.x as f64, max_p.y as f64, max_p.z as f64);
            BoundingBox::from_iter([bbx_min, bbx_max])
        };

        let mut current_angle = step_angle;
        let mut indx = 0;
        for i in 0..ANGLE_SUBDIVISIONS {
            let rot_for_points: Basis3<f32> = Rotation3::from_axis_angle(v, Rad(current_angle));
            let radius_dir2: Vector3<f32> = rot_for_points.rotate_vector(radius_dir1);
            let translation = Matrix4::from_translation(radius_dir2.mul(CABLE_EDGE_RADIUS));
            let p2: Point3<f32> = translation.transform_point(self.p_a);
            let p3: Point3<f32> = translation.transform_point(self.p_b);
            let rot_for_normal: Basis3<f32> = Rotation3::from_axis_angle(v, Rad(current_angle - step_half_angle));
            let normal: Vector3<f32> = -rot_for_normal.rotate_vector(radius_dir1);
            current_angle = current_angle + step_angle;

            vrtxs.push(p0.x);
            vrtxs.push(p0.y);
            vrtxs.push(p0.z);
            vrtxs.push(normal.x);
            vrtxs.push(normal.y);
            vrtxs.push(normal.z);
            indxs.push(indx);
            indx = indx + 1;

            vrtxs.push(p1.x);
            vrtxs.push(p1.y);
            vrtxs.push(p1.z);
            vrtxs.push(normal.x);
            vrtxs.push(normal.y);
            vrtxs.push(normal.z);
            indxs.push(indx);
            indx = indx + 1;

            vrtxs.push(p3.x);
            vrtxs.push(p3.y);
            vrtxs.push(p3.z);
            vrtxs.push(normal.x);
            vrtxs.push(normal.y);
            vrtxs.push(normal.z);
            indxs.push(indx);
            indx = indx + 1;

            vrtxs.push(p0.x);
            vrtxs.push(p0.y);
            vrtxs.push(p0.z);
            vrtxs.push(normal.x);
            vrtxs.push(normal.y);
            vrtxs.push(normal.z);
            indxs.push(indx);
            indx = indx + 1;

            vrtxs.push(p3.x);
            vrtxs.push(p3.y);
            vrtxs.push(p3.z);
            vrtxs.push(normal.x);
            vrtxs.push(normal.y);
            vrtxs.push(normal.z);
            indxs.push(indx);
            indx = indx + 1;

            vrtxs.push(p2.x);
            vrtxs.push(p2.y);
            vrtxs.push(p2.z);
            vrtxs.push(normal.x);
            vrtxs.push(normal.y);
            vrtxs.push(normal.z);
            indxs.push(indx);
            indx = indx + 1;


            let triangle0: Triangle =Triangle::new(
                Point3::new(p0.x,p0.y,p0.z),
                Point3::new(p1.x,p1.y,p1.z),
                Point3::new(p3.x,p3.y,p3.z),
            );
            let triangle1: Triangle =Triangle::new(
                Point3::new(p0.x,p0.y,p0.z),
                Point3::new(p3.x,p3.y,p3.z),
                Point3::new(p2.x,p2.y,p2.z),
            );
            triangles.push(triangle0);
            triangles.push(triangle1);
            p0 = p2;
            p1 = p3;
        }

        let rm = RawMesh {
            id: self.id,
            ty:PIPE_PRIME_TYPE,
            name: self.id.to_string(),
            vertex_normal: vrtxs,
            indx: indxs,
            color_indx: self.mat_idx,
            bbx: bbx,
            bvh_index: self.id as usize,
            triangles:triangles
        };
        rm
    }

    fn generate_perpendicular(v: Vector3<f32>) -> Vector3<f32> {
        let radius_dir1: Vector3<f32> = {
            let xr = v.x + 1000.0;
            let yr = v.y + 1000.0;
            let zr = -(v.x * xr + v.y * yr) / v.z;
            Vector3::new(xr, yr, zr).normalize()
        };

        if (v.is_perpendicular(radius_dir1)) {
            radius_dir1
        } else {
            let radius_dir2: Vector3<f32> = {
                let xr = v.x + 1000.0;
                let zr = v.z + 1000.0;
                let yr = -(v.x * xr + v.z * zr) / v.y;
                Vector3::new(xr, yr, zr).normalize()
            };
            if (v.is_perpendicular(radius_dir2)) {
                radius_dir2
            } else {
                let radius_dir3: Vector3<f32> = {
                    let yr = v.y + 1000.0;
                    let zr = v.z + 1000.0;
                    let xr = -(v.z * zr + v.y * yr) / v.x;
                    Vector3::new(xr, yr, zr).normalize()
                };
                radius_dir3
            }
        }
    }


}