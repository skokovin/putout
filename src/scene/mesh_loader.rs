use std::collections::HashMap;
use std::io::Cursor;
use std::ops::MulAssign;
use std::ptr;


use cgmath::Point3;
use itertools::Itertools;
use log::warn;
use miniz_oxide::inflate::decompress_to_vec;

use truck_base::bounding_box::BoundingBox;
use crate::scene::RawMesh;
use crate::shared::{CABLE_EDGE_COLOR, CABLE_EDGE_RADIUS, CABLE_NODE_COLOR, CABLE_NODE_SPHERE_RADIUS, Triangle};
use crate::shared::materials_lib::Material;
use crate::shared::mesh_common::MeshVertex;
use crate::shared::primitives_pipe::PipePrimitive;
use crate::shared::primitives_sphere::{SpherePrimitive};

pub const Z_FIGHTING_FACTOR: f32 = 1.0;
const TET: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDI: &[u8] = (include_bytes!("../dmp/hull/nf/data_ind.bin")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDV: &[u8] = (include_bytes!("../dmp/hull/nf/data_mesh.bin")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDB: &[u8] = (include_bytes!("../dmp/hull/nf/data_bbx.bin")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLID: &[u8] = (include_bytes!("../dmp/hull/nf/data_hash.bin")).as_slice();

#[cfg(target_arch = "wasm32")]
const HULLDI: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDV: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDB: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLID: &[u8] = &[0; 1];

pub fn read_hull_packed_new_format() -> (Vec<MeshVertex>, Vec<i32>, Vec<i32>, BoundingBox<Point3<f64>>, HashMap<i32, (i32, i32, i32)>, Vec<BoundingBox<Point3<f64>>>) {
    let decoded_mesh: Vec<u8> = decompress_to_vec(HULLDV).unwrap();
    let meshes_bytes_back: &[MeshVertex] = bytemuck::cast_slice(decoded_mesh.as_slice());

    let decoded_indxes: Vec<u8> = decompress_to_vec(HULLDI).unwrap();
    let indxes_bytes_back: &[i32] = bytemuck::cast_slice(decoded_indxes.as_slice());

    let decoded_bbxes: Vec<u8> = decompress_to_vec(HULLDB).unwrap();
    let bbxes_bytes_back: &[f32] = bytemuck::cast_slice(decoded_bbxes.as_slice());

    let mut out_bbx = {
        let pmin: Point3<f64> = Point3::new(-100.0, -100.0, -100.0);
        let pmax: Point3<f64> = Point3::new(100.0, 100.0, 100.0);
        let bbx = BoundingBox::from_iter([pmin, pmax]);
        bbx
    };
    let mut bbxes: Vec<BoundingBox<Point3<f64>>> = vec![];
    bbxes_bytes_back.chunks(6).for_each(|b| {
        let bbx: BoundingBox<Point3<f64>> = {
            let pmin: Point3<f64> = Point3::new(b[0] as f64, b[1] as f64, b[2] as f64);
            let pmax: Point3<f64> = Point3::new(b[3] as f64, b[4] as f64, b[5] as f64);
            let bbx = BoundingBox::from_iter([pmin, pmax]);
            bbx
        };
        out_bbx += &bbx;
        bbxes.push(bbx);
    });

    let mut meta_data: Vec<i32> = vec![];
    meshes_bytes_back.iter().for_each(|m| {
        meta_data.push(m.material_index);
    });

    let decoded_hashes: Vec<u8> = decompress_to_vec(HULLID).unwrap();
    let hashes_bytes_back: &[u32] = bytemuck::cast_slice(decoded_hashes.as_slice());

    let mut hull_mesh: HashMap<i32, (i32, i32, i32)> = HashMap::new();
    let mut counter = 0;
    hashes_bytes_back.chunks(3).for_each(|hash| {
        hull_mesh.insert(hash[0] as i32, (hash[1] as i32, hash[2] as i32, counter));
        counter = counter + 1;
    });

    (meshes_bytes_back.to_vec(), indxes_bytes_back.to_vec(), meta_data, out_bbx, hull_mesh, bbxes)
}

pub fn read_hull_unpacked_new_format(decoded_v: Vec<u8>, decoded_i: Vec<u8>, decoded_b: Vec<u8>, decoded_t: Vec<u8>) -> (Vec<MeshVertex>, Vec<i32>, Vec<i32>, BoundingBox<Point3<f64>>, HashMap<i32, (i32, i32, i32)>, Vec<BoundingBox<Point3<f64>>>) {
    warn!("start convert");

    let meshes_bytes_back: &[MeshVertex] = bytemuck::cast_slice(decoded_v.as_slice());
    let indxes_bytes_back: &[i32] = bytemuck::cast_slice(decoded_i.as_slice());
    let bbxes_bytes_back: &[f32] = bytemuck::cast_slice(decoded_b.as_slice());
    let mut out_bbx = {
        let pmin: Point3<f64> = Point3::new(-100.0, -100.0, -100.0);
        let pmax: Point3<f64> = Point3::new(100.0, 100.0, 100.0);
        let bbx = BoundingBox::from_iter([pmin, pmax]);
        bbx
    };
    let mut bbxes: Vec<BoundingBox<Point3<f64>>> = vec![];
    bbxes_bytes_back.chunks(6).for_each(|b| {
        let bbx: BoundingBox<Point3<f64>> = {
            let pmin: Point3<f64> = Point3::new(b[0] as f64, b[1] as f64, b[2] as f64);
            let pmax: Point3<f64> = Point3::new(b[3] as f64, b[4] as f64, b[5] as f64);
            let bbx = BoundingBox::from_iter([pmin, pmax]);
            bbx
        };
        out_bbx += &bbx;
        bbxes.push(bbx);
    });

    let mut meta_data: Vec<i32> = vec![];
    meshes_bytes_back.iter().for_each(|m| {
        //warn!("m.material_index {}",m.material_index);
        meta_data.push(m.material_index);
    });

    let hashes_bytes_back: &[u32] = bytemuck::cast_slice(decoded_t.as_slice());

    let mut hull_mesh: HashMap<i32, (i32, i32, i32)> = HashMap::new();
    let mut counter = 0;
    hashes_bytes_back.chunks(3).for_each(|hash| {
        hull_mesh.insert(hash[0] as i32, (hash[1] as i32, hash[2] as i32, counter));
        counter = counter + 1;
    });


    warn!("finish convert");
    (meshes_bytes_back.to_vec(), indxes_bytes_back.to_vec(), meta_data, out_bbx, hull_mesh, bbxes)
}

pub fn read_cable_with_test_data() -> Vec<RawMesh> {
    let mut out: Vec<RawMesh> = vec![];
    let p1: Point3<f32> = Point3::new(500.0, 0.0, 0.0);
    let s1: SpherePrimitive = SpherePrimitive::new(0, p1.clone(), CABLE_NODE_SPHERE_RADIUS, CABLE_NODE_COLOR);
    let rm1: RawMesh = s1.triangulate();
    out.push(rm1);

    let p2: Point3<f32> = Point3::new(700.0, 0.0, 0.0);
    let s2: SpherePrimitive = SpherePrimitive::new(1, p2.clone(), CABLE_NODE_SPHERE_RADIUS, CABLE_NODE_COLOR);
    let rm2: RawMesh = s2.triangulate();
    out.push(rm2);

    let edge1: PipePrimitive = PipePrimitive::new(3, p1, p2, CABLE_EDGE_RADIUS, CABLE_EDGE_COLOR);
    let rm4: RawMesh = edge1.triangulate();
    out.push(rm4);

    let p3: Point3<f32> = Point3::new(900.0, 0.0, 0.0);
    let s3: SpherePrimitive = SpherePrimitive::new(2, p3.clone(), CABLE_NODE_SPHERE_RADIUS, CABLE_NODE_COLOR);
    let rm3: RawMesh = s3.triangulate();
    out.push(rm3);

    let edge2: PipePrimitive = PipePrimitive::new(4, p2, p3, CABLE_EDGE_RADIUS, CABLE_EDGE_COLOR);
    let rm5: RawMesh = edge2.triangulate();
    out.push(rm5);

    out
}
