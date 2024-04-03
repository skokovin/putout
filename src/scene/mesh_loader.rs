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

    //let mesh : &[u8] = (include_bytes!("../dmp/hull/nf/data_mesh.bin")).as_slice();
    let decoded_mesh: Vec<u8> = decompress_to_vec(HULLDV).unwrap();
    let meshes_bytes_back: &[MeshVertex] =bytemuck::cast_slice(decoded_mesh.as_slice());

    //let indxes : &[u8] = (include_bytes!("../dmp/hull/nf/data_ind.bin")).as_slice();
    let decoded_indxes: Vec<u8> = decompress_to_vec(HULLDI).unwrap();
    let indxes_bytes_back: &[i32] =bytemuck::cast_slice(decoded_indxes.as_slice());

    //let bbxes : &[u8] = (include_bytes!("../dmp/hull/nf/data_bbx.bin")).as_slice();
    let decoded_bbxes: Vec<u8> = decompress_to_vec(HULLDB).unwrap();
    let bbxes_bytes_back: &[f32] =bytemuck::cast_slice(decoded_bbxes.as_slice());

    let mut out_bbx = {
        let pmin: Point3<f64> = Point3::new(-100.0, -100.0, -100.0);
        let pmax: Point3<f64> = Point3::new(100.0, 100.0, 100.0);
        let bbx = BoundingBox::from_iter([pmin, pmax]);
        bbx
    };
    let mut bbxes:Vec<BoundingBox<Point3<f64>>>=vec![];
    bbxes_bytes_back.chunks(6).for_each(|b|{
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
    meshes_bytes_back.iter().for_each(|m|{
        //warn!("m.material_index {}",m.material_index);
        meta_data.push(m.material_index);

    });


    //let hashes : &[u8] = (include_bytes!("../dmp/hull/nf/data_hash.bin")).as_slice();
    let decoded_hashes: Vec<u8> = decompress_to_vec(HULLID).unwrap();
    let hashes_bytes_back: &[u32] =bytemuck::cast_slice(decoded_hashes.as_slice());

    let mut hull_mesh: HashMap<i32, (i32, i32,i32)>= HashMap::new();
    let mut counter=0;
    hashes_bytes_back.chunks(3).for_each(|hash|{
        hull_mesh.insert(hash[0] as i32, (hash[1] as i32, hash[2] as i32,counter));
        counter=counter+1;
    });

    (meshes_bytes_back.to_vec(),indxes_bytes_back.to_vec(),meta_data,out_bbx,hull_mesh,bbxes)
}

pub fn read_hull_unpacked_new_format(decoded_v: Vec<u8>, decoded_i: Vec<u8>, decoded_b: Vec<u8>, decoded_t: Vec<u8>) -> (Vec<MeshVertex>, Vec<i32>, Vec<i32>, BoundingBox<Point3<f64>>, HashMap<i32, (i32, i32, i32)>, Vec<BoundingBox<Point3<f64>>>) {
    warn!("start convert");

    let meshes_bytes_back: &[MeshVertex] =bytemuck::cast_slice(decoded_v.as_slice());
    let indxes_bytes_back: &[i32] =bytemuck::cast_slice(decoded_i.as_slice());
    let bbxes_bytes_back: &[f32] =bytemuck::cast_slice(decoded_b.as_slice());
    let mut out_bbx = {
        let pmin: Point3<f64> = Point3::new(-100.0, -100.0, -100.0);
        let pmax: Point3<f64> = Point3::new(100.0, 100.0, 100.0);
        let bbx = BoundingBox::from_iter([pmin, pmax]);
        bbx
    };
    let mut bbxes:Vec<BoundingBox<Point3<f64>>>=vec![];
    bbxes_bytes_back.chunks(6).for_each(|b|{
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
    meshes_bytes_back.iter().for_each(|m|{
        //warn!("m.material_index {}",m.material_index);
        meta_data.push(m.material_index);

    });

    let hashes_bytes_back: &[u32] =bytemuck::cast_slice(decoded_t.as_slice());

    let mut hull_mesh: HashMap<i32, (i32, i32,i32)>= HashMap::new();
    let mut counter=0;
    hashes_bytes_back.chunks(3).for_each(|hash|{
        hull_mesh.insert(hash[0] as i32, (hash[1] as i32, hash[2] as i32,counter));
        counter=counter+1;
    });


    warn!("finish convert");
    (meshes_bytes_back.to_vec(),indxes_bytes_back.to_vec(),meta_data,out_bbx,hull_mesh,bbxes)
}
pub fn read_hull_packed() -> Vec<RawMesh> {
    println!("unpack started");
    let decoded_v: Vec<u8> = decompress_to_vec(HULLDV).unwrap();
    let decoded_i: Vec<u8> = decompress_to_vec(HULLDI).unwrap();
    let decoded_b: Vec<u8> = decompress_to_vec(HULLDB).unwrap();
    let decoded_t: Vec<u8> = decompress_to_vec(HULLID).unwrap();
    println!("decoded_v {} decoded_i {}  decoded_b {}", decoded_v.len() / 4, decoded_i.len() / 4, decoded_b.len() / 4);
    read_unpacked(decoded_v, decoded_i, decoded_b, decoded_t)
}
pub fn read_unpacked(decoded_v: Vec<u8>, decoded_i: Vec<u8>, decoded_b: Vec<u8>, decoded_t: Vec<u8>) -> Vec<RawMesh> {
    warn!("start convert");
    let mut meshes: Vec<RawMesh> = vec![];
    let mut bounding_boxes: Vec<BoundingBox<Point3<f64>>> = vec![];
    let mut out_i: Vec<i32> = Vec::from(bytemuck::cast_slice(&decoded_i));
    let mut out_v: Vec<f32> = Vec::from(bytemuck::cast_slice(&decoded_v));
    let mut out_b: Vec<f32> = Vec::from(bytemuck::cast_slice(&decoded_b));
    let mut out_t: Vec<i32> = Vec::from(bytemuck::cast_slice(&decoded_t));
    let ids: Vec<&[i32]> = out_t.chunks(2).collect_vec();
    let vertexes = out_v.chunks(6).collect_vec();
    out_b.chunks(6).for_each(|c| {
        let p_min = Point3::new(c[0] as f64 * Z_FIGHTING_FACTOR as f64, c[1] as f64 * Z_FIGHTING_FACTOR as f64, c[2] as f64 * Z_FIGHTING_FACTOR as f64);
        let p_max = Point3::new(c[3] as f64 * Z_FIGHTING_FACTOR as f64, c[4] as f64 * Z_FIGHTING_FACTOR as f64, c[5] as f64 * Z_FIGHTING_FACTOR as f64);
        let bbx: BoundingBox<Point3<f64>> = BoundingBox::from_iter([p_min, p_max]);
        bounding_boxes.push(bbx);
    });

    let mut global_counter = 0;
    let mut mesh_counter = 0;

    let mut vt: Vec<f32> = vec![];
    let mut idx: Vec<i32> = vec![];
    let mut triangles: Vec<Triangle> = vec![];
    warn!("start foreach");
    out_i.chunks(3).for_each(|tri| {
        let si = tri[0].clone();
        if (si == 0 && vt.len() != 0) {
            let id: i32 = ids[mesh_counter][0];
            let ty: i32 = ids[mesh_counter][1];
            let rm = RawMesh {
                id: id,
                ty: ty,
                name: mesh_counter.to_string(),
                vertex_normal: vt.clone(),
                indx: idx.clone(),
                color_indx: Material::type_to_color(ty),
                bbx: bounding_boxes[mesh_counter].clone(),
                bvh_index: id as usize,
                triangles: triangles.clone(),
            };
            meshes.push(rm);
            mesh_counter = mesh_counter + 1;
            vt = vec![];
            idx = vec![];
            triangles = vec![];
        }
        let v0 = vertexes[global_counter];
        vt.push(v0[0] * Z_FIGHTING_FACTOR);
        vt.push(v0[1] * Z_FIGHTING_FACTOR);
        vt.push(v0[2] * Z_FIGHTING_FACTOR);
        vt.push(v0[3]);
        vt.push(v0[4]);
        vt.push(v0[5]);
        idx.push(tri[0].clone());
        global_counter = global_counter + 1;

        let v1 = vertexes[global_counter];
        vt.push(v1[0] * Z_FIGHTING_FACTOR);
        vt.push(v1[1] * Z_FIGHTING_FACTOR);
        vt.push(v1[2] * Z_FIGHTING_FACTOR);
        vt.push(v1[3]);
        vt.push(v1[4]);
        vt.push(v1[5]);
        idx.push(tri[1].clone());
        global_counter = global_counter + 1;

        let v2 = vertexes[global_counter];
        vt.push(v2[0] * Z_FIGHTING_FACTOR);
        vt.push(v2[1] * Z_FIGHTING_FACTOR);
        vt.push(v2[2] * Z_FIGHTING_FACTOR);
        vt.push(v2[3]);
        vt.push(v2[4]);
        vt.push(v2[5]);
        idx.push(tri[2].clone());
        global_counter = global_counter + 1;

        let triangle: Triangle = Triangle::new(
            Point3::new(v0[0] * Z_FIGHTING_FACTOR, v0[1] * Z_FIGHTING_FACTOR, v0[2] * Z_FIGHTING_FACTOR),
            Point3::new(v1[0] * Z_FIGHTING_FACTOR, v1[1] * Z_FIGHTING_FACTOR, v1[2] * Z_FIGHTING_FACTOR),
            Point3::new(v2[0] * Z_FIGHTING_FACTOR, v2[1] * Z_FIGHTING_FACTOR, v2[2] * Z_FIGHTING_FACTOR),
        );
        triangles.push(triangle);
    });
    warn!("finish foreach");
    let id: i32 = ids[mesh_counter][0];
    let ty: i32 = ids[mesh_counter][1];
    let rm = RawMesh {
        id: id,
        ty: ty,
        name: mesh_counter.to_string(),
        vertex_normal: vt.clone(),
        indx: idx.clone(),
        color_indx: Material::type_to_color(ty),
        bbx: bounding_boxes[mesh_counter].clone(),
        bvh_index: id as usize,
        triangles: triangles.clone(),
    };
    meshes.push(rm);
    mesh_counter = mesh_counter + 1;
    warn!("end convert");
    meshes
}
pub fn read_unpacked_wasm(decoded_v: Vec<u8>, decoded_i: Vec<u8>, decoded_b: Vec<u8>, decoded_t: Vec<u8>) -> Vec<RawMesh> {
    warn!("start convert");
    let mut meshes: Vec<RawMesh> = vec![];
    let mut bounding_boxes: Vec<BoundingBox<Point3<f64>>> = vec![];
    let  out_i: Vec<i32> = Vec::from(bytemuck::cast_slice(&decoded_i));
    let  out_v: Vec<f32> = Vec::from(bytemuck::cast_slice(&decoded_v));
    let  out_b: Vec<f32> = Vec::from(bytemuck::cast_slice(&decoded_b));
    let  out_t: Vec<i32> = Vec::from(bytemuck::cast_slice(&decoded_t));
    let ids: Vec<&[i32]> = out_t.chunks(2).collect_vec();
    let vertexes = out_v.chunks(6).collect_vec();
    out_b.chunks(6).for_each(|c| {
        let p_min = Point3::new(c[0] as f64 * Z_FIGHTING_FACTOR as f64, c[1] as f64 * Z_FIGHTING_FACTOR as f64, c[2] as f64 * Z_FIGHTING_FACTOR as f64);
        let p_max = Point3::new(c[3] as f64 * Z_FIGHTING_FACTOR as f64, c[4] as f64 * Z_FIGHTING_FACTOR as f64, c[5] as f64 * Z_FIGHTING_FACTOR as f64);
        let bbx: BoundingBox<Point3<f64>> = BoundingBox::from_iter([p_min, p_max]);
        bounding_boxes.push(bbx);
    });

    let mut global_counter = 0;
    let mut mesh_counter = 0;

    let mut vt: Vec<f32> = vec![];
    let mut idx: Vec<i32> = vec![];
    let mut triangles: Vec<Triangle> = vec![];
    warn!("start foreach");
    out_i.chunks(3).for_each(|tri| {
        let si = tri[0].clone();
        if (si == 0 && vt.len() != 0) {
            let id: i32 = ids[mesh_counter][0];
            let ty: i32 = ids[mesh_counter][1];
            let rm = RawMesh {
                id: id,
                ty: ty,
                name: mesh_counter.to_string(),
                vertex_normal: vt.clone(),
                indx: idx.clone(),
                color_indx: Material::type_to_color(ty),
                bbx: bounding_boxes[mesh_counter].clone(),
                bvh_index: id as usize,
                triangles: triangles.clone(),
            };
            //if(mesh_counter<20000){
                meshes.push(rm);
            //}

            mesh_counter = mesh_counter + 1;
            vt = vec![];
            idx = vec![];
            triangles = vec![];
        }
        let v0 = vertexes[global_counter];
        vt.push(v0[0] * Z_FIGHTING_FACTOR);
        vt.push(v0[1] * Z_FIGHTING_FACTOR);
        vt.push(v0[2] * Z_FIGHTING_FACTOR);
        vt.push(v0[3]);
        vt.push(v0[4]);
        vt.push(v0[5]);
        idx.push(tri[0].clone());
        global_counter = global_counter + 1;

        let v1 = vertexes[global_counter];
        vt.push(v1[0] * Z_FIGHTING_FACTOR);
        vt.push(v1[1] * Z_FIGHTING_FACTOR);
        vt.push(v1[2] * Z_FIGHTING_FACTOR);
        vt.push(v1[3]);
        vt.push(v1[4]);
        vt.push(v1[5]);
        idx.push(tri[1].clone());
        global_counter = global_counter + 1;

        let v2 = vertexes[global_counter];
        vt.push(v2[0] * Z_FIGHTING_FACTOR);
        vt.push(v2[1] * Z_FIGHTING_FACTOR);
        vt.push(v2[2] * Z_FIGHTING_FACTOR);
        vt.push(v2[3]);
        vt.push(v2[4]);
        vt.push(v2[5]);
        idx.push(tri[2].clone());
        global_counter = global_counter + 1;

        let triangle: Triangle = Triangle::new(
            Point3::new(v0[0] * Z_FIGHTING_FACTOR, v0[1] * Z_FIGHTING_FACTOR, v0[2] * Z_FIGHTING_FACTOR),
            Point3::new(v1[0] * Z_FIGHTING_FACTOR, v1[1] * Z_FIGHTING_FACTOR, v1[2] * Z_FIGHTING_FACTOR),
            Point3::new(v2[0] * Z_FIGHTING_FACTOR, v2[1] * Z_FIGHTING_FACTOR, v2[2] * Z_FIGHTING_FACTOR),
        );
        triangles.push(triangle);
    });
    warn!("finish foreach");

    meshes
}

pub fn read_unpacked_wasm_orig(decoded_v: Vec<u8>, decoded_i: Vec<u8>, decoded_b: Vec<u8>, decoded_t: Vec<u8>) -> Vec<RawMesh> {
    warn!("start convert");
    let mut meshes: Vec<RawMesh> = vec![];
    let mut bounding_boxes: Vec<BoundingBox<Point3<f64>>> = vec![];
    let mut out_i: Vec<i32> = Vec::from(bytemuck::cast_slice(&decoded_i));
    let mut out_v: Vec<f32> = Vec::from(bytemuck::cast_slice(&decoded_v));
    let mut out_b: Vec<f32> = Vec::from(bytemuck::cast_slice(&decoded_b));
    let mut out_t: Vec<i32> = Vec::from(bytemuck::cast_slice(&decoded_t));
    let ids: Vec<&[i32]> = out_t.chunks(2).collect_vec();
    let vertexes = out_v.chunks(6).collect_vec();
    out_b.chunks(6).for_each(|c| {
        let p_min = Point3::new(c[0] as f64 * Z_FIGHTING_FACTOR as f64, c[1] as f64 * Z_FIGHTING_FACTOR as f64, c[2] as f64 * Z_FIGHTING_FACTOR as f64);
        let p_max = Point3::new(c[3] as f64 * Z_FIGHTING_FACTOR as f64, c[4] as f64 * Z_FIGHTING_FACTOR as f64, c[5] as f64 * Z_FIGHTING_FACTOR as f64);
        let bbx: BoundingBox<Point3<f64>> = BoundingBox::from_iter([p_min, p_max]);
        bounding_boxes.push(bbx);
    });

    let mut global_counter = 0;
    let mut mesh_counter = 0;

    let mut vt: Vec<f32> = vec![];
    let mut idx: Vec<i32> = vec![];
    let mut triangles: Vec<Triangle> = vec![];
    warn!("start foreach");
    out_i.chunks(3).for_each(|tri| {
        let si = tri[0].clone();
        if (si == 0 && vt.len() != 0) {
            let id: i32 = ids[mesh_counter][0];
            let ty: i32 = ids[mesh_counter][1];
            let rm = RawMesh {
                id: id,
                ty: ty,
                name: mesh_counter.to_string(),
                vertex_normal: vt.clone(),
                indx: idx.clone(),
                color_indx: Material::type_to_color(ty),
                bbx: bounding_boxes[mesh_counter].clone(),
                bvh_index: id as usize,
                triangles: triangles.clone(),
            };
            meshes.push(rm);
            mesh_counter = mesh_counter + 1;
            vt = vec![];
            idx = vec![];
            triangles = vec![];
        }
        let v0 = vertexes[global_counter];
        vt.push(v0[0] * Z_FIGHTING_FACTOR);
        vt.push(v0[1] * Z_FIGHTING_FACTOR);
        vt.push(v0[2] * Z_FIGHTING_FACTOR);
        vt.push(v0[3]);
        vt.push(v0[4]);
        vt.push(v0[5]);
        idx.push(tri[0].clone());
        global_counter = global_counter + 1;

        let v1 = vertexes[global_counter];
        vt.push(v1[0] * Z_FIGHTING_FACTOR);
        vt.push(v1[1] * Z_FIGHTING_FACTOR);
        vt.push(v1[2] * Z_FIGHTING_FACTOR);
        vt.push(v1[3]);
        vt.push(v1[4]);
        vt.push(v1[5]);
        idx.push(tri[1].clone());
        global_counter = global_counter + 1;

        let v2 = vertexes[global_counter];
        vt.push(v2[0] * Z_FIGHTING_FACTOR);
        vt.push(v2[1] * Z_FIGHTING_FACTOR);
        vt.push(v2[2] * Z_FIGHTING_FACTOR);
        vt.push(v2[3]);
        vt.push(v2[4]);
        vt.push(v2[5]);
        idx.push(tri[2].clone());
        global_counter = global_counter + 1;

        let triangle: Triangle = Triangle::new(
            Point3::new(v0[0] * Z_FIGHTING_FACTOR, v0[1] * Z_FIGHTING_FACTOR, v0[2] * Z_FIGHTING_FACTOR),
            Point3::new(v1[0] * Z_FIGHTING_FACTOR, v1[1] * Z_FIGHTING_FACTOR, v1[2] * Z_FIGHTING_FACTOR),
            Point3::new(v2[0] * Z_FIGHTING_FACTOR, v2[1] * Z_FIGHTING_FACTOR, v2[2] * Z_FIGHTING_FACTOR),
        );
        triangles.push(triangle);
    });
    warn!("finish foreach");
    let id: i32 = ids[mesh_counter][0];
    let ty: i32 = ids[mesh_counter][1];
    let rm = RawMesh {
        id: id,
        ty: ty,
        name: mesh_counter.to_string(),
        vertex_normal: vt.clone(),
        indx: idx.clone(),
        color_indx: Material::type_to_color(ty),
        bbx: bounding_boxes[mesh_counter].clone(),
        bvh_index: id as usize,
        triangles: triangles.clone(),
    };
    meshes.push(rm);
    mesh_counter = mesh_counter + 1;
    warn!("end convert");
    meshes
}

/*pub fn read_unpacked_orig(decoded_v: Vec<u8>, decoded_i: Vec<u8>, decoded_b: Vec<u8>, decoded_t: Vec<u8>) -> Vec<RawMesh> {
    let mut meshes: Vec<RawMesh> = vec![];
    let mut bounding_boxes: Vec<BoundingBox<Point3<f64>>> = vec![];
    warn!("copy started");
    let mut out_i: Vec<i32> = Vec::from(bytemuck::cast_slice(&decoded_i));
    let mut out_v: Vec<f32> = Vec::from(bytemuck::cast_slice(&decoded_v));
    let mut out_b: Vec<f32> = Vec::from(bytemuck::cast_slice(&decoded_b));
    let mut out_t: Vec<i32> = Vec::from(bytemuck::cast_slice(&decoded_t));
    let ids: Vec<&[i32]> = out_t.chunks(2).collect_vec();
    warn!("copy finish");

    out_b.chunks(6).for_each(|c| {
        let p_min = Point3::new(c[0] as f64 * Z_FIGHTING_FACTOR as f64, c[1] as f64 * Z_FIGHTING_FACTOR as f64, c[2] as f64 * Z_FIGHTING_FACTOR as f64);
        let p_max = Point3::new(c[3] as f64 * Z_FIGHTING_FACTOR as f64, c[4] as f64 * Z_FIGHTING_FACTOR as f64, c[5] as f64 * Z_FIGHTING_FACTOR as f64);
        let bbx: BoundingBox<Point3<f64>> = BoundingBox::from_iter([p_min, p_max]);
        bounding_boxes.push(bbx);
    });
    warn!("A");
    let mut iii: Vec<Vec<i32>> = vec![];
    let mut ii: Vec<i32> = vec![];
    ii.push(out_i[0]);
    out_i[1..].iter().for_each(|i| {
        if (*i == 0) {
            iii.push(ii.clone());
            ii = vec![];
        }
        ii.push(*i);
    });
    iii.push(ii.clone());
    warn!("B");
    let vertexes = out_v.chunks(6).collect_vec();

    let mut global_counter = 0;
    let mut mesh_counter = 0;

    iii.iter().for_each(|ii| {
        let bbx = bounding_boxes[mesh_counter].clone();
        let mut vt: Vec<f32> = vec![];
        let mut idx: Vec<i32> = vec![];
        let mut triangles: Vec<Triangle<f32>> = vec![];
        ii.chunks(3).for_each(|tri| {
            let v0 = vertexes[global_counter];
            vt.push(v0[0] * Z_FIGHTING_FACTOR);
            vt.push(v0[1] * Z_FIGHTING_FACTOR);
            vt.push(v0[2] * Z_FIGHTING_FACTOR);
            vt.push(v0[3]);
            vt.push(v0[4]);
            vt.push(v0[5]);
            idx.push(tri[0].clone());
            global_counter = global_counter + 1;

            let v1 = vertexes[global_counter];
            vt.push(v1[0] * Z_FIGHTING_FACTOR);
            vt.push(v1[1] * Z_FIGHTING_FACTOR);
            vt.push(v1[2] * Z_FIGHTING_FACTOR);
            vt.push(v1[3]);
            vt.push(v1[4]);
            vt.push(v1[5]);
            idx.push(tri[1].clone());
            global_counter = global_counter + 1;

            let v2 = vertexes[global_counter];
            vt.push(v2[0] * Z_FIGHTING_FACTOR);
            vt.push(v2[1] * Z_FIGHTING_FACTOR);
            vt.push(v2[2] * Z_FIGHTING_FACTOR);
            vt.push(v2[3]);
            vt.push(v2[4]);
            vt.push(v2[5]);
            idx.push(tri[2].clone());
            global_counter = global_counter + 1;

            let triangle: Triangle<f32> = Triangle::new(
                triangle::Point::new(v0[0] * Z_FIGHTING_FACTOR, v0[1] * Z_FIGHTING_FACTOR, v0[2] * Z_FIGHTING_FACTOR),
                triangle::Point::new(v1[0] * Z_FIGHTING_FACTOR, v1[1] * Z_FIGHTING_FACTOR, v1[2] * Z_FIGHTING_FACTOR),
                triangle::Point::new(v2[0] * Z_FIGHTING_FACTOR, v2[1] * Z_FIGHTING_FACTOR, v2[2] * Z_FIGHTING_FACTOR),
            );
            triangles.push(triangle);
        });

        let id: i32 = ids[mesh_counter][0];
        let ty: i32 = ids[mesh_counter][1];
        let rm = RawMesh {
            id: id,
            ty: ty,
            name: mesh_counter.to_string(),
            vertex_normal: vt.clone(),
            indx: idx.clone(),
            color_indx: Material::type_to_color(ty),
            bbx: bbx,
            bvh_index: id as usize,
            triangles: triangles,
        };
        meshes.push(rm);
        mesh_counter = mesh_counter + 1;
    });
    warn!("C+ {}", meshes.len());
    meshes
}
*/
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
