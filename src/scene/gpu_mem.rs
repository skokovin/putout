use std::collections::HashMap;
use std::mem::size_of;
use std::rc::Rc;
use cgmath::Point3;
use log::warn;
use parking_lot::RwLock;
use truck_base::bounding_box::BoundingBox;
use web_sys::js_sys::Uint8Array;
use wgpu::{Buffer, Device};
use wgpu::util::DeviceExt;
use crate::device::message_controller::ActionType;

use crate::scene::scene_state::SceneState;
use crate::shared::materials_lib::{HIDDEN_HULL_MAT, Material, SELECTION_HULL_MAT};
use crate::shared::mesh_common::MeshVertex;
use crate::shared::Triangle;
#[cfg(target_arch = "wasm32")]
use crate::remote::hull_state::get_mesh_vertex_by_id;
pub const ID_MEM_OFFSET: u32 = 100;

pub struct GpuMem {
    device: Rc<RwLock<Device>>,
    pub id: u32,
    pub tot_loc_bbx: BoundingBox<Point3<f64>>,
    pub loc_bbxs: Vec<BoundingBox<Point3<f64>>>,
    pub mesh_hash: HashMap<i32, (i32, i32, i32)>,
    pub v: Vec<MeshVertex>,
    pub i: Vec<i32>,
    pub metadata: Vec<i32>,
    pub is_metadata_dirty: bool,
    pub v_buffer: Buffer,
    pub i_buffer: Buffer,
    pub is_renderable: bool,
}

impl GpuMem {
    pub fn new(device: Rc<RwLock<Device>>, id: u32) -> Self {
        let tot_loc_bbx = {
            let pmin: Point3<f64> = Point3::new(-1000.0, -1000.0, -1000.0);
            let pmax: Point3<f64> = Point3::new(1000.0, 1000.0, 1000.0);
            let bbx = BoundingBox::from_iter([pmin, pmax]);
            bbx
        };
        let indicies: Vec<i32> = vec![];
        let index_buffer = device.clone().read().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("Index Mesh Buffer {id}").as_str()),
            contents: bytemuck::cast_slice(&indicies),
            usage: wgpu::BufferUsages::INDEX,
        });

        let vertexes: Vec<MeshVertex> = vec![];
        let vertex_buffer = device.clone().read().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("Vertex Mesh Buffer {id}").as_str()),
            contents: bytemuck::cast_slice(&vertexes),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Self {
            device: device,
            id: id,
            tot_loc_bbx,
            loc_bbxs: vec![],
            mesh_hash: HashMap::new(),
            v: vec![],
            i: vec![],
            metadata: vec![],
            is_metadata_dirty: false,
            v_buffer: vertex_buffer,
            i_buffer: index_buffer,
            is_renderable: false,
        }
    }
    pub fn resize_buffers(&mut self) {
        self.i_buffer = self.device.read().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("Index Mesh Buffer {}", self.id).as_str()),
            contents: bytemuck::cast_slice(&self.i),
            usage: wgpu::BufferUsages::INDEX,
        });
        self.v_buffer = self.device.read().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("Vertex Mesh Buffer {}", self.id).as_str()),
            contents: bytemuck::cast_slice(&self.v),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.is_renderable = true;
    }
    pub fn set_data(&mut self, v: Vec<MeshVertex>, i: Vec<i32>, metadata: Vec<i32>, tot_loc_bbx: BoundingBox<Point3<f64>>, mesh_hash: HashMap<i32, (i32, i32, i32)>, loc_bbxs: Vec<BoundingBox<Point3<f64>>>) {
        warn!("MEMORY SIZE IS {} {}", size_of::<MeshVertex>()*v.len(),  size_of::<i32>()*i.len() );
        self.v = v;
        self.i = i;
        self.metadata = metadata;
        //self.loc_bbxs = loc_bbxs;
        self.tot_loc_bbx = tot_loc_bbx;
        self.mesh_hash = mesh_hash;
        self.resize_buffers();
        self.is_metadata_dirty = true;
        #[cfg(target_arch = "wasm32")]
        {
            self.i = vec![];
            self.v = vec![];
        }
    }

    pub fn select_by_id(&mut self, oid: i32) -> bool {
        match self.mesh_hash.get(&oid) {
            None => {
                false
            }
            Some(mesh) => {
                self.change_material(SELECTION_HULL_MAT, mesh.0, mesh.1);
                true
            }
        }
    }
    pub fn hide_by_id(&mut self, oid: i32) -> bool {
        match self.mesh_hash.get(&oid) {
            None => {
                false
            }
            Some(mesh) => {
                self.change_material(HIDDEN_HULL_MAT, mesh.0, mesh.1);
                true
            }
        }
    }
    pub fn set_default_by_id(&mut self, oid: i32) -> bool {
        match self.mesh_hash.get(&oid) {
            None => {
                false
            }
            Some(mesh) => {
                self.change_material(self.get_default_material_by_id(oid), mesh.0, mesh.1);
                true
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn get_triangle_by_vertex_index(&self, vertex_index: usize) -> Option<(i32, Triangle)> {
        let bin = get_mesh_vertex_by_id(self.id as i32, vertex_index as i32).to_vec();
        let meshes: &[MeshVertex] = bytemuck::cast_slice(bin.as_slice());

        if(meshes.len()>0){
           let base_mesh=meshes[0];
           match self.mesh_hash.get(&base_mesh.id) {
                None => { None }
                Some(_mesh_object) => {
                    let local_base_trangle: Triangle = {
                        match (vertex_index + 1) % 3 {
                            0 => {
                                let v0 = &self.get_mesh_by_id(vertex_index);
                                let v1 = &self.get_mesh_by_id(vertex_index+1);
                                let v2 = &self.get_mesh_by_id(vertex_index+2);
                                Triangle::from_coords(
                                    v0.position[0], v0.position[1], v0.position[2],
                                    v1.position[0], v1.position[1], v1.position[2],
                                    v2.position[0], v2.position[1], v2.position[2],
                                )
                            }
                            1 => {
                                let v0 =&self.get_mesh_by_id(vertex_index-1);
                                let v1 = &self.get_mesh_by_id(vertex_index);
                                let v2 =&self.get_mesh_by_id(vertex_index+1);
                                Triangle::from_coords(
                                    v0.position[0], v0.position[1], v0.position[2],
                                    v1.position[0], v1.position[1], v1.position[2],
                                    v2.position[0], v2.position[1], v2.position[2],
                                )
                            }
                            2 => {
                                let v0 = &self.v[vertex_index - 2];
                                let v1 = &self.v[vertex_index - 1];
                                let v2 = &self.v[vertex_index];
                                Triangle::from_coords(
                                    v0.position[0], v0.position[1], v0.position[2],
                                    v1.position[0], v1.position[1], v1.position[2],
                                    v2.position[0], v2.position[1], v2.position[2],
                                )
                            }
                            _ => {
                                warn!("SOMETING GOES WRONG!!");
                                let v0 = &self.get_mesh_by_id(vertex_index);
                                let v1 = &self.get_mesh_by_id(vertex_index+1);
                                let v2 = &self.get_mesh_by_id(vertex_index+2);
                                Triangle::from_coords(
                                    v0.position[0], v0.position[1], v0.position[2],
                                    v1.position[0], v1.position[1], v1.position[2],
                                    v2.position[0], v2.position[1], v2.position[2],
                                )
                            }
                        }
                    };
                    Some((base_mesh.id, local_base_trangle.clone()))
                }
            }

        }else{
            None
        }



    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_triangle_by_vertex_index(&self, vertex_index: usize) -> Option<(i32, Triangle)> {
        match self.v.get(vertex_index) {
            None => { None }
            Some(base_mesh) => {
                match self.mesh_hash.get(&base_mesh.id) {
                    None => { None }
                    Some(_mesh_object) => {
                        let local_base_trangle: Triangle = {
                            match (vertex_index + 1) % 3 {
                                0 => {
                                    let v0 = &self.v[vertex_index];
                                    let v1 = &self.v[vertex_index + 1];
                                    let v2 = &self.v[vertex_index + 2];
                                    Triangle::from_coords(
                                        v0.position[0], v0.position[1], v0.position[2],
                                        v1.position[0], v1.position[1], v1.position[2],
                                        v2.position[0], v2.position[1], v2.position[2],
                                    )
                                }
                                1 => {
                                    let v0 = &self.v[vertex_index - 1];
                                    let v1 = &self.v[vertex_index];
                                    let v2 = &self.v[vertex_index + 1];
                                    Triangle::from_coords(
                                        v0.position[0], v0.position[1], v0.position[2],
                                        v1.position[0], v1.position[1], v1.position[2],
                                        v2.position[0], v2.position[1], v2.position[2],
                                    )
                                }
                                2 => {
                                    let v0 = &self.v[vertex_index - 2];
                                    let v1 = &self.v[vertex_index - 1];
                                    let v2 = &self.v[vertex_index];
                                    Triangle::from_coords(
                                        v0.position[0], v0.position[1], v0.position[2],
                                        v1.position[0], v1.position[1], v1.position[2],
                                        v2.position[0], v2.position[1], v2.position[2],
                                    )
                                }
                                _ => {
                                    warn!("SOMETING GOES WRONG!!");
                                    let v0 = &self.v[vertex_index];
                                    let v1 = &self.v[vertex_index + 1];
                                    let v2 = &self.v[vertex_index + 2];
                                    Triangle::from_coords(
                                        v0.position[0], v0.position[1], v0.position[2],
                                        v1.position[0], v1.position[1], v1.position[2],
                                        v2.position[0], v2.position[1], v2.position[2],
                                    )
                                }
                            }
                        };
                        Some((base_mesh.id, local_base_trangle.clone()))
                    }
                }
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    fn get_mesh_by_id(&self,vertex_index:usize)->MeshVertex{
        let bin = get_mesh_vertex_by_id(self.id as i32, vertex_index as i32).to_vec();
        let meshes: &[MeshVertex] = bytemuck::cast_slice(bin.as_slice());
        meshes[0]
    }
    pub fn get_bbx_by_oid(&self, oid: i32) -> Option<BoundingBox<Point3<f64>>> {
        match self.mesh_hash.get(&oid) {
            None => { None }
            Some(mesh) => {
                let indx = mesh.2;
                match self.loc_bbxs.get(indx as usize) {
                    None => { None }
                    Some(bbx) => { Some(bbx.clone()) }
                }
            }
        }
    }

    fn change_material(&mut self, mat_indx: i32, start: i32, end: i32) {
        self.metadata[start as usize..=end as usize].iter_mut().for_each(|m| {
            *m = mat_indx;
        });
        self.is_metadata_dirty = true;
    }
    #[cfg(target_arch = "wasm32")]
    fn get_default_material_by_id(&self, id: i32) -> i32 {
        match self.mesh_hash.get(&id) {
            None => { 0 }
            Some(m) => {
                let start_index = m.1;
                let bin = get_mesh_vertex_by_id(self.id as i32, start_index).to_vec();
                let meshes: &[MeshVertex] = bytemuck::cast_slice(bin.as_slice());
                if (meshes.len() > 0) {
                    let mesh_v = meshes[0];
                    let default_material = Material::type_to_color(unpack_id(mesh_v.material_index as u32) as i32);
                    default_material
                } else {
                    0
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn get_default_material_by_id(&self, id: i32) -> i32 {
        match self.mesh_hash.get(&id) {
            None => { 0 }
            Some(m) => {
                let start_index = m.1;
                let mesh_v = self.v[start_index as usize];
                let default_material = Material::type_to_color(unpack_id(mesh_v.material_index as u32) as i32);
                default_material
            }
        }
    }


    pub fn reset_dirty_metadata(&mut self) {
        self.is_metadata_dirty = false;
    }
}


pub fn unpack_id(raw_id: u32) -> u32 {
    let pack_id = raw_id % ID_MEM_OFFSET;
    let id = (raw_id - pack_id) / ID_MEM_OFFSET;
    id
}

pub fn unpack_packid(raw_id: u32) -> u32 {
    let pack_id = raw_id % ID_MEM_OFFSET;
    pack_id
}