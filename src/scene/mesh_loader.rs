use std::collections::HashMap;





use cgmath::Point3;
use itertools::Itertools;
use log::warn;
use miniz_oxide::inflate::decompress_to_vec;

use truck_base::bounding_box::BoundingBox;
use crate::scene::gpu_mem::unpack_id;
use crate::scene::RawMesh;
use crate::shared::{CABLE_EDGE_COLOR, CABLE_EDGE_RADIUS, CABLE_NODE_COLOR, CABLE_NODE_SPHERE_RADIUS};

use crate::shared::mesh_common::MeshVertex;
use crate::shared::primitives_pipe::PipePrimitive;
use crate::shared::primitives_sphere::{SpherePrimitive};

pub const Z_FIGHTING_FACTOR: f32 = 1.0;
const TET: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDI0: &[u8] = (include_bytes!("../dmp/hull/nf/0data_ind")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDV0: &[u8] = (include_bytes!("../dmp/hull/nf/0data_mesh")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDB0: &[u8] = (include_bytes!("../dmp/hull/nf/0data_bbx")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLID0: &[u8] = (include_bytes!("../dmp/hull/nf/0data_hash")).as_slice();

#[cfg(not(target_arch = "wasm32"))]
const HULLDI1: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDV1: &[u8] =&[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDB1: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLID1: &[u8] = &[0; 1];

#[cfg(not(target_arch = "wasm32"))]
const HULLDI2: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDV2: &[u8] =&[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDB2: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLID2: &[u8] = &[0; 1];

#[cfg(not(target_arch = "wasm32"))]
const HULLDI3: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDV3: &[u8] =&[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDB3: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLID3: &[u8] = &[0; 1];

#[cfg(not(target_arch = "wasm32"))]
const HULLDI4: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDV4: &[u8] =&[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDB4: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLID4: &[u8] = &[0; 1];

#[cfg(not(target_arch = "wasm32"))]
const HULLDI5: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDV5: &[u8] =&[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDB5: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLID5: &[u8] = &[0; 1];

#[cfg(not(target_arch = "wasm32"))]
const HULLDI6: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDV6: &[u8] =&[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDB6: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLID6: &[u8] = &[0; 1];

#[cfg(not(target_arch = "wasm32"))]
const HULLDI7: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDV7: &[u8] =&[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLDB7: &[u8] = &[0; 1];
#[cfg(not(target_arch = "wasm32"))]
const HULLID7: &[u8] = &[0; 1];


/*#[cfg(not(target_arch = "wasm32"))]
const HULLDI1: &[u8] = (include_bytes!("../dmp/hull/nf/1data_ind")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDV1: &[u8] = (include_bytes!("../dmp/hull/nf/1data_mesh")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDB1: &[u8] = (include_bytes!("../dmp/hull/nf/1data_bbx")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLID1: &[u8] = (include_bytes!("../dmp/hull/nf/1data_hash")).as_slice();

#[cfg(not(target_arch = "wasm32"))]
const HULLDI2: &[u8] = (include_bytes!("../dmp/hull/nf/2data_ind")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDV2: &[u8] = (include_bytes!("../dmp/hull/nf/2data_mesh")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDB2: &[u8] = (include_bytes!("../dmp/hull/nf/2data_bbx")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLID2: &[u8] = (include_bytes!("../dmp/hull/nf/2data_hash")).as_slice();


#[cfg(not(target_arch = "wasm32"))]
const HULLDI3: &[u8] = (include_bytes!("../dmp/hull/nf/3data_ind")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDV3: &[u8] = (include_bytes!("../dmp/hull/nf/3data_mesh")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDB3: &[u8] = (include_bytes!("../dmp/hull/nf/3data_bbx")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLID3: &[u8] = (include_bytes!("../dmp/hull/nf/3data_hash")).as_slice();


#[cfg(not(target_arch = "wasm32"))]
const HULLDI4: &[u8] = (include_bytes!("../dmp/hull/nf/4data_ind")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDV4: &[u8] = (include_bytes!("../dmp/hull/nf/4data_mesh")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDB4: &[u8] = (include_bytes!("../dmp/hull/nf/4data_bbx")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLID4: &[u8] = (include_bytes!("../dmp/hull/nf/4data_hash")).as_slice();

#[cfg(not(target_arch = "wasm32"))]
const HULLDI5: &[u8] = (include_bytes!("../dmp/hull/nf/5data_ind")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDV5: &[u8] = (include_bytes!("../dmp/hull/nf/5data_mesh")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDB5: &[u8] = (include_bytes!("../dmp/hull/nf/5data_bbx")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLID5: &[u8] = (include_bytes!("../dmp/hull/nf/5data_hash")).as_slice();

#[cfg(not(target_arch = "wasm32"))]
const HULLDI6: &[u8] = (include_bytes!("../dmp/hull/nf/6data_ind")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDV6: &[u8] = (include_bytes!("../dmp/hull/nf/6data_mesh")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDB6: &[u8] = (include_bytes!("../dmp/hull/nf/6data_bbx")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLID6: &[u8] = (include_bytes!("../dmp/hull/nf/6data_hash")).as_slice();


#[cfg(not(target_arch = "wasm32"))]
const HULLDI7: &[u8] = (include_bytes!("../dmp/hull/nf/7data_ind")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDV7: &[u8] = (include_bytes!("../dmp/hull/nf/7data_mesh")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLDB7: &[u8] = (include_bytes!("../dmp/hull/nf/7data_bbx")).as_slice();
#[cfg(not(target_arch = "wasm32"))]
const HULLID7: &[u8] = (include_bytes!("../dmp/hull/nf/7data_hash")).as_slice();*/

#[cfg(target_arch = "wasm32")]
const HULLDI0: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDV0: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDB0: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLID0: &[u8] = &[0; 1];

#[cfg(target_arch = "wasm32")]
const HULLDI1: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDV1: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDB1: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLID1: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDI2: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDV2: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDB2: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLID2: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDI3: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDV3: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDB3: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLID3: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDI4: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDV4: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDB4: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLID4: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDI5: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDV5: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDB5: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLID5: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDI6: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDV6: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDB6: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLID6: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDI7: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDV7: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLDB7: &[u8] = &[0; 1];
#[cfg(target_arch = "wasm32")]
const HULLID7: &[u8] = &[0; 1];

pub fn read_hull_packed_new_format0() -> (Vec<MeshVertex>, Vec<i32>, Vec<i32>, BoundingBox<Point3<f64>>, HashMap<i32, (i32, i32, i32)>, Vec<BoundingBox<Point3<f64>>>) {
    let decoded_mesh: Vec<u8> = decompress_to_vec(HULLDV0).unwrap();
    let meshes_bytes_back: &[MeshVertex] = bytemuck::cast_slice(decoded_mesh.as_slice());

    let decoded_indxes: Vec<u8> = decompress_to_vec(HULLDI0).unwrap();
    let indxes_bytes_back: &[i32] = bytemuck::cast_slice(decoded_indxes.as_slice());

    let decoded_bbxes: Vec<u8> = decompress_to_vec(HULLDB0).unwrap();
    let bbxes_bytes_back: &[f32] = bytemuck::cast_slice(decoded_bbxes.as_slice());

    let mut out_bbx = {
        let pmin: Point3<f64> = Point3::new(-100.0, -100.0, -100.0);
        let pmax: Point3<f64> = Point3::new(100.0, 100.0, 100.0);
        let bbx = BoundingBox::from_iter([pmin, pmax]);
        bbx
    };
    let mut bbxes: Vec<BoundingBox<Point3<f64>>> = Vec::with_capacity(bbxes_bytes_back.len()/2);
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

    let mut meta_data: Vec<i32> =   Vec::with_capacity(meshes_bytes_back.len());
    meshes_bytes_back.iter().for_each(|m| {
        meta_data.push(unpack_id(m.material_index as u32) as i32);
    });

    let decoded_hashes: Vec<u8> = decompress_to_vec(HULLID0).unwrap();
    let hashes_bytes_back: &[u32] = bytemuck::cast_slice(decoded_hashes.as_slice());

    let mut hull_mesh: HashMap<i32, (i32, i32, i32)> = HashMap::new();
    let mut counter = 0;
    hashes_bytes_back.chunks(3).for_each(|hash| {
        hull_mesh.insert(hash[0] as i32, (hash[1] as i32, hash[2] as i32, counter));
        counter = counter + 1;
    });

    (meshes_bytes_back.to_vec(), indxes_bytes_back.to_vec(), meta_data, out_bbx, hull_mesh, bbxes)
}
pub fn read_hull_packed_new_format1() -> (Vec<MeshVertex>, Vec<i32>, Vec<i32>, BoundingBox<Point3<f64>>, HashMap<i32, (i32, i32, i32)>, Vec<BoundingBox<Point3<f64>>>) {
    let decoded_mesh: Vec<u8> = decompress_to_vec(HULLDV1).unwrap();
    let meshes_bytes_back: &[MeshVertex] = bytemuck::cast_slice(decoded_mesh.as_slice());

    let decoded_indxes: Vec<u8> = decompress_to_vec(HULLDI1).unwrap();
    let indxes_bytes_back: &[i32] = bytemuck::cast_slice(decoded_indxes.as_slice());

    let decoded_bbxes: Vec<u8> = decompress_to_vec(HULLDB1).unwrap();
    let bbxes_bytes_back: &[f32] = bytemuck::cast_slice(decoded_bbxes.as_slice());

    let mut out_bbx = {
        let pmin: Point3<f64> = Point3::new(-100.0, -100.0, -100.0);
        let pmax: Point3<f64> = Point3::new(100.0, 100.0, 100.0);
        let bbx = BoundingBox::from_iter([pmin, pmax]);
        bbx
    };
    let mut bbxes: Vec<BoundingBox<Point3<f64>>> = Vec::with_capacity(bbxes_bytes_back.len()/2);
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

    let mut meta_data: Vec<i32> =   Vec::with_capacity(meshes_bytes_back.len());
    meshes_bytes_back.iter().for_each(|m| {
        meta_data.push(unpack_id(m.material_index as u32) as i32);
    });

    let decoded_hashes: Vec<u8> = decompress_to_vec(HULLID1).unwrap();
    let hashes_bytes_back: &[u32] = bytemuck::cast_slice(decoded_hashes.as_slice());

    let mut hull_mesh: HashMap<i32, (i32, i32, i32)> = HashMap::new();
    let mut counter = 0;
    hashes_bytes_back.chunks(3).for_each(|hash| {
        hull_mesh.insert(hash[0] as i32, (hash[1] as i32, hash[2] as i32, counter));
        counter = counter + 1;
    });

    (meshes_bytes_back.to_vec(), indxes_bytes_back.to_vec(), meta_data, out_bbx, hull_mesh, bbxes)
}
pub fn read_hull_packed_new_format2() -> (Vec<MeshVertex>, Vec<i32>, Vec<i32>, BoundingBox<Point3<f64>>, HashMap<i32, (i32, i32, i32)>, Vec<BoundingBox<Point3<f64>>>) {
    let decoded_mesh: Vec<u8> = decompress_to_vec(HULLDV2).unwrap();
    let meshes_bytes_back: &[MeshVertex] = bytemuck::cast_slice(decoded_mesh.as_slice());

    let decoded_indxes: Vec<u8> = decompress_to_vec(HULLDI2).unwrap();
    let indxes_bytes_back: &[i32] = bytemuck::cast_slice(decoded_indxes.as_slice());

    let decoded_bbxes: Vec<u8> = decompress_to_vec(HULLDB2).unwrap();
    let bbxes_bytes_back: &[f32] = bytemuck::cast_slice(decoded_bbxes.as_slice());

    let mut out_bbx = {
        let pmin: Point3<f64> = Point3::new(-100.0, -100.0, -100.0);
        let pmax: Point3<f64> = Point3::new(100.0, 100.0, 100.0);
        let bbx = BoundingBox::from_iter([pmin, pmax]);
        bbx
    };
    let mut bbxes: Vec<BoundingBox<Point3<f64>>> = Vec::with_capacity(bbxes_bytes_back.len()/2);
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

    let mut meta_data: Vec<i32> =  Vec::with_capacity(meshes_bytes_back.len());
    meshes_bytes_back.iter().for_each(|m| {
        meta_data.push(unpack_id(m.material_index as u32) as i32);
    });

    let decoded_hashes: Vec<u8> = decompress_to_vec(HULLID2).unwrap();
    let hashes_bytes_back: &[u32] = bytemuck::cast_slice(decoded_hashes.as_slice());

    let mut hull_mesh: HashMap<i32, (i32, i32, i32)> = HashMap::new();
    let mut counter = 0;
    hashes_bytes_back.chunks(3).for_each(|hash| {
        hull_mesh.insert(hash[0] as i32, (hash[1] as i32, hash[2] as i32, counter));
        counter = counter + 1;
    });

    (meshes_bytes_back.to_vec(), indxes_bytes_back.to_vec(), meta_data, out_bbx, hull_mesh, bbxes)
}
pub fn read_hull_packed_new_format3() -> (Vec<MeshVertex>, Vec<i32>, Vec<i32>, BoundingBox<Point3<f64>>, HashMap<i32, (i32, i32, i32)>, Vec<BoundingBox<Point3<f64>>>) {
    let decoded_mesh: Vec<u8> = decompress_to_vec(HULLDV3).unwrap();
    let meshes_bytes_back: &[MeshVertex] = bytemuck::cast_slice(decoded_mesh.as_slice());
    let decoded_indxes: Vec<u8> = decompress_to_vec(HULLDI3).unwrap();
    let indxes_bytes_back: &[i32] = bytemuck::cast_slice(decoded_indxes.as_slice());
    let decoded_bbxes: Vec<u8> = decompress_to_vec(HULLDB3).unwrap();
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
        meta_data.push(unpack_id(m.material_index as u32) as i32);
    });
    let decoded_hashes: Vec<u8> = decompress_to_vec(HULLID3).unwrap();
    let hashes_bytes_back: &[u32] = bytemuck::cast_slice(decoded_hashes.as_slice());
    let mut hull_mesh: HashMap<i32, (i32, i32, i32)> = HashMap::new();
    let mut counter = 0;
    hashes_bytes_back.chunks(3).for_each(|hash| {
        hull_mesh.insert(hash[0] as i32, (hash[1] as i32, hash[2] as i32, counter));
        counter = counter + 1;
    });
    (meshes_bytes_back.to_vec(), indxes_bytes_back.to_vec(), meta_data, out_bbx, hull_mesh, bbxes)
}
pub fn read_hull_packed_new_format4() -> (Vec<MeshVertex>, Vec<i32>, Vec<i32>, BoundingBox<Point3<f64>>, HashMap<i32, (i32, i32, i32)>, Vec<BoundingBox<Point3<f64>>>) {
    let decoded_mesh: Vec<u8> = decompress_to_vec(HULLDV4).unwrap();
    let meshes_bytes_back: &[MeshVertex] = bytemuck::cast_slice(decoded_mesh.as_slice());
    let decoded_indxes: Vec<u8> = decompress_to_vec(HULLDI4).unwrap();
    let indxes_bytes_back: &[i32] = bytemuck::cast_slice(decoded_indxes.as_slice());
    let decoded_bbxes: Vec<u8> = decompress_to_vec(HULLDB4).unwrap();
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
        meta_data.push(unpack_id(m.material_index as u32) as i32);
    });
    let decoded_hashes: Vec<u8> = decompress_to_vec(HULLID4).unwrap();
    let hashes_bytes_back: &[u32] = bytemuck::cast_slice(decoded_hashes.as_slice());
    let mut hull_mesh: HashMap<i32, (i32, i32, i32)> = HashMap::new();
    let mut counter = 0;
    hashes_bytes_back.chunks(3).for_each(|hash| {
        hull_mesh.insert(hash[0] as i32, (hash[1] as i32, hash[2] as i32, counter));
        counter = counter + 1;
    });
    (meshes_bytes_back.to_vec(), indxes_bytes_back.to_vec(), meta_data, out_bbx, hull_mesh, bbxes)
}
pub fn read_hull_packed_new_format5() -> (Vec<MeshVertex>, Vec<i32>, Vec<i32>, BoundingBox<Point3<f64>>, HashMap<i32, (i32, i32, i32)>, Vec<BoundingBox<Point3<f64>>>) {
    let decoded_mesh: Vec<u8> = decompress_to_vec(HULLDV5).unwrap();
    let meshes_bytes_back: &[MeshVertex] = bytemuck::cast_slice(decoded_mesh.as_slice());
    let decoded_indxes: Vec<u8> = decompress_to_vec(HULLDI5).unwrap();
    let indxes_bytes_back: &[i32] = bytemuck::cast_slice(decoded_indxes.as_slice());
    let decoded_bbxes: Vec<u8> = decompress_to_vec(HULLDB5).unwrap();
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
        meta_data.push(unpack_id(m.material_index as u32) as i32);
    });
    let decoded_hashes: Vec<u8> = decompress_to_vec(HULLID5).unwrap();
    let hashes_bytes_back: &[u32] = bytemuck::cast_slice(decoded_hashes.as_slice());
    let mut hull_mesh: HashMap<i32, (i32, i32, i32)> = HashMap::new();
    let mut counter = 0;
    hashes_bytes_back.chunks(3).for_each(|hash| {
        hull_mesh.insert(hash[0] as i32, (hash[1] as i32, hash[2] as i32, counter));
        counter = counter + 1;
    });
    (meshes_bytes_back.to_vec(), indxes_bytes_back.to_vec(), meta_data, out_bbx, hull_mesh, bbxes)
}
pub fn read_hull_packed_new_format6() -> (Vec<MeshVertex>, Vec<i32>, Vec<i32>, BoundingBox<Point3<f64>>, HashMap<i32, (i32, i32, i32)>, Vec<BoundingBox<Point3<f64>>>) {
    let decoded_mesh: Vec<u8> = decompress_to_vec(HULLDV6).unwrap();
    let meshes_bytes_back: &[MeshVertex] = bytemuck::cast_slice(decoded_mesh.as_slice());
    let decoded_indxes: Vec<u8> = decompress_to_vec(HULLDI6).unwrap();
    let indxes_bytes_back: &[i32] = bytemuck::cast_slice(decoded_indxes.as_slice());
    let decoded_bbxes: Vec<u8> = decompress_to_vec(HULLDB6).unwrap();
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
        meta_data.push(unpack_id(m.material_index as u32) as i32);
    });
    let decoded_hashes: Vec<u8> = decompress_to_vec(HULLID6).unwrap();
    let hashes_bytes_back: &[u32] = bytemuck::cast_slice(decoded_hashes.as_slice());
    let mut hull_mesh: HashMap<i32, (i32, i32, i32)> = HashMap::new();
    let mut counter = 0;
    hashes_bytes_back.chunks(3).for_each(|hash| {
        hull_mesh.insert(hash[0] as i32, (hash[1] as i32, hash[2] as i32, counter));
        counter = counter + 1;
    });
    (meshes_bytes_back.to_vec(), indxes_bytes_back.to_vec(), meta_data, out_bbx, hull_mesh, bbxes)
}
pub fn read_hull_packed_new_format7() -> (Vec<MeshVertex>, Vec<i32>, Vec<i32>, BoundingBox<Point3<f64>>, HashMap<i32, (i32, i32, i32)>, Vec<BoundingBox<Point3<f64>>>) {
    let decoded_mesh: Vec<u8> = decompress_to_vec(HULLDV7).unwrap();
    let meshes_bytes_back: &[MeshVertex] = bytemuck::cast_slice(decoded_mesh.as_slice());
    let decoded_indxes: Vec<u8> = decompress_to_vec(HULLDI7).unwrap();
    let indxes_bytes_back: &[i32] = bytemuck::cast_slice(decoded_indxes.as_slice());
    let decoded_bbxes: Vec<u8> = decompress_to_vec(HULLDB7).unwrap();
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
        meta_data.push(unpack_id(m.material_index as u32) as i32);
    });
    let decoded_hashes: Vec<u8> = decompress_to_vec(HULLID7).unwrap();
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
    //warn!("start convert");

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
        meta_data.push(unpack_id(m.material_index as u32) as i32);
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

pub fn read_hull_unpacked_new_format_remote(load_level:i32,decoded_v: &[u8], decoded_i: &[u8], decoded_b: &[u8], decoded_t: &[u8]) -> (Vec<MeshVertex>, Vec<i32>, Vec<i32>, BoundingBox<Point3<f64>>, HashMap<i32, (i32, i32, i32)>, Vec<BoundingBox<Point3<f64>>>) {
    warn!("start convert remote");

    let meshes_bytes_back: &[MeshVertex] = bytemuck::cast_slice(decoded_v);
    let indxes_bytes_back: &[i32] = bytemuck::cast_slice(decoded_i);
    let bbxes_bytes_back: &[f32] = bytemuck::cast_slice(decoded_b);
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
        //meta_data.push(m.material_index);
        meta_data.push(unpack_id(m.material_index as u32) as i32);
    });

    let hashes_bytes_back: &[u32] = bytemuck::cast_slice(decoded_t);

    let mut hull_mesh: HashMap<i32, (i32, i32, i32)> = HashMap::new();
    let mut counter = 0;
    hashes_bytes_back.chunks(3).for_each(|hash| {
        hull_mesh.insert(hash[0] as i32, (hash[1] as i32, hash[2] as i32, counter));
        counter = counter + 1;
    });


    //warn!("finish convert");
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
