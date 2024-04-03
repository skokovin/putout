use std::fmt::{Debug, Formatter};
use cgmath::{Point3, Vector3};

use truck_base::bounding_box::BoundingBox;
use crate::shared::Triangle;

pub mod scene_state;
pub mod mesh_loader;
pub mod scene_details;

#[derive(Clone)]
pub struct RawMesh {
    pub id: i32,
    pub ty: i32,
    pub name: String,
    pub vertex_normal: Vec<f32>,
    pub indx: Vec<i32>,
    pub color_indx: i32,
    pub bbx: BoundingBox<Point3<f64>>,
    pub bvh_index: usize,
    pub triangles: Vec<Triangle>,

}


impl Default for RawMesh {
    fn default() -> Self {
        RawMesh {
            id: -999,
            ty: -999,
            name: "NONE".to_string(),
            vertex_normal: vec![],
            indx: vec![],
            color_indx: -99,
            bbx: BoundingBox::default(),
            bvh_index: 0,
            triangles: vec![],
        }
    }
}

impl Debug for RawMesh {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RawMesh: ID={} NAME={}  TYPE={} MAT={} BBX={:?} V={} I={} BVH={}",
               self.id, self.name, self.ty, self.color_indx, self.bbx, self.vertex_normal.len(), self.indx.len(), self.bvh_index)
    }
}