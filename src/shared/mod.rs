use std::ops::Sub;
use cgmath::{Point3, Vector3};
use serde::{Deserialize, Serialize};


pub mod ship_consts;
pub mod materials_lib;
pub mod mesh_pipeline;
pub mod primitives_sphere;

pub mod mesh_common;
pub mod wasm_remote;
pub mod highlight_pipeline;
pub mod primitives_pipe;
pub mod shared_buffers;
pub mod screen_capture;
pub mod text_layout;
pub mod dimension;

pub const SPHERE_PRIME_TYPE: i32 = 700;
pub const PIPE_PRIME_TYPE: i32 = 701;
pub const CABLE_NODE_SPHERE_RADIUS: f32 = 5.0;
pub const CABLE_EDGE_RADIUS: f32 = 1.0;
pub const CABLE_NODE_COLOR: i32 = 2;
pub const CABLE_EDGE_COLOR: i32 = 2;
pub const ANGLE_SUBDIVISIONS: u32 = 8;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TriMesh {
    indices: Vec<i32>,
    positions: Vec<Vector3<f32>>,
    normals: Vec<Vector3<f32>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Triangle {
    pub p: [Point3<f32>; 3],
    pub normal: Vector3<f32>,
}

impl Triangle {
    pub fn new(p0: Point3<f32>, p1: Point3<f32>, p2: Point3<f32>) -> Self {
        let u = p1.sub(p0);
        let v = p2.sub(p0);
        let normal: Vector3<f32> = Vector3::new(
            u.y * v.z - u.z * v.y,
            u.z * v.x - u.x * v.z,
            u.x * v.y - u.y * v.x,
        );
        Self {
            p: [p0, p1, p2],
            normal: normal,
        }
    }

    pub fn from_coords(
        x0: f32, y0: f32, z0: f32,
        x1: f32, y1: f32, z1: f32,
        x2: f32, y2: f32, z2: f32,
    ) -> Self {
        let p0: Point3<f32> = Point3::new(x0, y0, z0);
        let p1: Point3<f32> = Point3::new(x1, y1, z1);
        let p2: Point3<f32> = Point3::new(x2, y2, z2);
        Triangle::new(p0, p1, p2)
    }
}




