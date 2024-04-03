use std::collections::{HashMap, HashSet};
use std::mem;
use std::mem::size_of;
use std::ops::Sub;
use std::rc::Rc;
use cgmath::{InnerSpace, Point3, Vector3};
use cgmath::num_traits::Float;
use itertools::Itertools;
use log::{info, warn};
use parking_lot::RwLock;
use rand::Rng;

use truck_base::bounding_box::{Bounded, BoundingBox};
use truck_polymesh::Attributes;

use wgpu::{Buffer, BufferAddress, Device};
use crate::device::message_controller::{ActionType, MessageController};
use crate::scene::{mesh_loader, RawMesh};
use crate::shared::mesh_common::MeshVertex;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalPosition;
use winit::event::DeviceId;
use crate::gui::camera_base::{CameraBase, SHIP_FORWARD};
use crate::gui::slicer::Slicer;
use crate::remote::hull_state;
use crate::shared::materials_lib::{HIDDEN_HULL_MAT, Material, SELECTION_HULL_MAT};
use crate::shared::Triangle;

pub struct SceneState {
    device: Rc<RwLock<Device>>,
    default_bbx: BoundingBox<Point3<f64>>,
    pub tot_bbx: BoundingBox<Point3<f64>>,
    pub bbxs:Vec<BoundingBox<Point3<f64>>>,

    pub camera: CameraBase,
    pub slicer: Slicer,

    pub hull_mesh: HashMap<i32, (i32, i32, i32)>,
    selected_hull_ids: HashSet<i32>,
    hidden_hull_ids: HashSet<i32>,
    pub hull_v: Vec<MeshVertex>,
    hull_i: Vec<i32>,
    pub hull_metadata: Vec<i32>,
    pub is_hull_metadata_dirty: bool,
    pub hull_vertex_buffer: Buffer,
    pub hull_index_buffer: Buffer,

    pub is_snap_dirty: bool,
    pub snap_vertex_buffer: Buffer,

}

impl SceneState {
    pub fn new(device: Rc<RwLock<Device>>) -> Self {
        let default_bbx = {
            let pmin: Point3<f64> = Point3::new(-1000.0, -1000.0, -1000.0);
            let pmax: Point3<f64> = Point3::new(1000.0, 1000.0, 1000.0);
            let bbx = BoundingBox::from_iter([pmin, pmax]);
            bbx
        };
        let hull_indirect_indicies: Vec<i32> = vec![];
        let hull_index_buffer = device.clone().read().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer Hull"),
            contents: bytemuck::cast_slice(&hull_indirect_indicies),
            usage: wgpu::BufferUsages::INDEX,
        });
        let hull_indirect_vertexes: Vec<MeshVertex> = vec![];
        let hull_vertex_buffer = device.clone().read().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer Hull"),
            contents: bytemuck::cast_slice(&hull_indirect_vertexes),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let cable_indirect_indicies: Vec<i32> = vec![];
        let cable_index_buffer = device.clone().read().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer cable"),
            contents: bytemuck::cast_slice(&cable_indirect_indicies),
            usage: wgpu::BufferUsages::INDEX,
        });
        let cable_indirect_vertexes: Vec<MeshVertex> = vec![];
        let cable_vertex_buffer = device.clone().read().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer cable"),
            contents: bytemuck::cast_slice(&cable_indirect_vertexes),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let snap_vertex_buffer: Buffer = device.clone().read().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Snap Vertex"),
            size: 128,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });


        Self {
            device: device,
            default_bbx,
            tot_bbx: Default::default(),
            bbxs:vec![],
            camera: CameraBase::default(),
            slicer: Slicer::default(),
            hull_mesh: HashMap::new(),
            selected_hull_ids: HashSet::new(),
            hidden_hull_ids: HashSet::new(),
            hull_v: vec![],
            hull_i: vec![],
            hull_metadata: vec![],
            is_hull_metadata_dirty: false,
            hull_vertex_buffer: hull_vertex_buffer,
            hull_index_buffer: hull_index_buffer,
            is_snap_dirty: false,
            snap_vertex_buffer: snap_vertex_buffer,
        }
    }

    pub fn set_hull_mesh(&mut self) {
        let (hm, i, meta_data, out_bbx, hull_mesh,bbxs) = mesh_loader::read_hull_packed_new_format();
        self.hull_v = hm;
        self.hull_i = i;
        self.hull_metadata = meta_data;
        self.bbxs=bbxs;
        self.tot_bbx = out_bbx;
        self.hull_mesh = hull_mesh;
        self.slicer.set_by_bbx(&self.tot_bbx);
        self.resize_hull_buffers();
        self.is_hull_metadata_dirty = true;
        println!("SIZE {}", self.hull_i.len());
    }

    pub fn set_hull_mesh_orug(&mut self) {
        let mut hull_mesh: Vec<RawMesh> = mesh_loader::read_hull_packed();
        hull_mesh.iter().for_each(|mesh| {
            self.tot_bbx += &mesh.bbx;
            //println!("{} {}",index,mesh.id);
            //self.hull_mesh.insert(mesh.id, (0, 0, mesh.clone()));
        });
        println!("SIZE {}", self.hull_mesh.len());
        //self.send_hull_to_device();
    }

    pub fn set_hull_mesh_remote(&mut self, decoded_v: Vec<u8>, decoded_i: Vec<u8>, decoded_b: Vec<u8>, decoded_t: Vec<u8>) {
        warn!("TRY LOAD HULL FROm REMOTE");
        let (hm, i, meta_data, out_bbx, hull_mesh,bbxs)
            = mesh_loader::read_hull_unpacked_new_format( decoded_v, decoded_i, decoded_b, decoded_t);
        self.hull_v = hm;
        self.hull_i = i;
        self.hull_metadata = meta_data;
        self.bbxs=bbxs;
        self.tot_bbx = out_bbx;
        self.hull_mesh = hull_mesh;
        self.slicer.set_by_bbx(&self.tot_bbx);
        self.resize_hull_buffers();
        self.is_hull_metadata_dirty = true;
        println!("SIZE {}", self.hull_i.len());
    }


/*    fn send_hull_to_device(&mut self) {
        self.slicer.set_by_bbx(&self.tot_bbx);
        let (v, i, md) = self.convert_hull_parts_to_buffers();
        self.hull_metadata = md;
        self.hull_v.extend_from_slice(v.as_slice());
        self.hull_i.extend_from_slice(i.as_slice());
        info!("V {} I {}",v.len(), i.len() );
        self.resize_hull_buffers();
        self.is_hull_metadata_dirty = true;
        info!("BBX x{} y{} z{} X{} Y{} Z{}", self.tot_bbx.min().x, self.tot_bbx.min().y, self.tot_bbx.min().z
        , self.tot_bbx.max().x, self.tot_bbx.max().y, self.tot_bbx.max().z)
    }*/
    pub fn reset_dirty_hull_metadata(&mut self) {
        self.is_hull_metadata_dirty = false;
    }

    pub fn screen_oid(&mut self, action: ActionType, oid: i32) -> bool {
        let mut is_scene_modified = false;
        if (oid != 0) {
            match self.hull_v.get(oid as usize) {
                None => {
                    is_scene_modified
                }
                Some(mesh) => {
                    match action {
                        ActionType::Select => {
                            self.select_hull_by_id(mesh.id);
                            self.refresh_hull_remote_selected();
                            is_scene_modified
                        }
                        ActionType::Hide => {
                            self.hide_hull_by_id(mesh.id);
                            self.refresh_hull_remote_hidden();
                            is_scene_modified = true;
                            is_scene_modified
                        }
                        ActionType::Evaluate => {
                            is_scene_modified
                        }
                    }
                }
            }
        } else {
            is_scene_modified
        }
    }
    /*    fn mesh_analyzis(&mut self, vertex_index: i32, mesh: MeshVertex) -> Triangle {
            let hm = self.hull_mesh.get(&mesh.id).unwrap();
            //let raw_mesh = &hm.2;
            let triangles = &raw_mesh.triangles;
            let start_index = hm.0 as usize;
            let end_index = hm.1 as usize;
            let vertexes=&self.hull_v[start_index..end_index+1];


            let mut points: Vec<Point3<f32>> = vec![];

            raw_mesh.vertex_normal.chunks(6).for_each(|ch| {
                points.push(Point3::new(ch[0], ch[1], ch[2]));
            });
            let local_index = (vertex_index - start_index) / 3;
            let local_point = points[local_index as usize];
            let local_trangle = triangles[local_index as usize].clone();

            let global_point = Point3::new(
                mesh.position[0],
                mesh.position[1],
                mesh.position[2]);
            //warn!("start_index={} end_index={} vertex_index={}", start_index,end_index,vertex_index);
            //warn!("local_index={} local_point={:?} vertex_index={:?}", local_index,local_point,global_point);
            //warn!("TRIANGLE={:?} of {}", local_trangle,triangles.len() );

            local_trangle
        }
    */
    pub fn get_hull_triangle_by_index(&self, index: usize) -> Option<(i32, Triangle)> {
        if (index < 2) {
            None
        } else {
            match self.hull_v.get(index) {
                None => { None }
                Some(base_mesh) => {
                    match self.hull_mesh.get(&base_mesh.id) {
                        None => { None }
                        Some(mesh_object) => {
                            //let start_index = mesh_object.0 as usize;
                            //let end_index = mesh_object.1 as usize;
                            //let vertexes = &self.hull_v[start_index..end_index + 1];
                            let local_base_trangle: Triangle = {
                                match (index + 1) % 3 {
                                    0 => {
                                        let v0 = &self.hull_v[index];
                                        let v1 = &self.hull_v[index + 1];
                                        let v2 = &self.hull_v[index + 2];
                                        Triangle::from_coords(
                                            v0.position[0], v0.position[1], v0.position[2],
                                            v1.position[0], v1.position[1], v1.position[2],
                                            v2.position[0], v2.position[1], v2.position[2],
                                        )
                                    }
                                    1 => {
                                        let v0 = &self.hull_v[index - 1];
                                        let v1 = &self.hull_v[index];
                                        let v2 = &self.hull_v[index + 1];
                                        Triangle::from_coords(
                                            v0.position[0], v0.position[1], v0.position[2],
                                            v1.position[0], v1.position[1], v1.position[2],
                                            v2.position[0], v2.position[1], v2.position[2],
                                        )
                                    }
                                    2 => {
                                        let v0 = &self.hull_v[index - 2];
                                        let v1 = &self.hull_v[index - 1];
                                        let v2 = &self.hull_v[index];
                                        Triangle::from_coords(
                                            v0.position[0], v0.position[1], v0.position[2],
                                            v1.position[0], v1.position[1], v1.position[2],
                                            v2.position[0], v2.position[1], v2.position[2],
                                        )
                                    }
                                    _ => {
                                        warn!("SOMETING GOES WRONG!!");
                                        let v0 = &self.hull_v[index];
                                        let v1 = &self.hull_v[index + 1];
                                        let v2 = &self.hull_v[index + 2];
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
    }
    pub fn on_render(&mut self) { self.camera.update(self.tot_bbx.clone()) }
    pub fn select_hull_by_ids(&mut self, ids: HashSet<i32>) {
        self.unselect_hull_all();
        ids.iter().for_each(|id| {
            match self.hull_mesh.get(id) {
                None => {}
                Some(mesh) => {
                    self.change_hull_material(SELECTION_HULL_MAT, mesh.0, mesh.1);
                    self.selected_hull_ids.insert(id.clone());
                }
            }
        });
    }
    pub fn select_hull_by_id(&mut self, oid: i32) {
        match self.selected_hull_ids.get(&oid) {
            None => {
                self.unselect_hull_all();
                match self.hull_mesh.get(&oid) {
                    None => {}
                    Some(mesh) => {
                        self.change_hull_material(SELECTION_HULL_MAT, mesh.0, mesh.1);
                        self.selected_hull_ids.insert(oid);
                    }
                }
            }
            Some(part) => {
                self.unselect_hull_all();
            }
        }
    }
    pub fn select_hull_by_part(&mut self, rm: RawMesh) {
        match self.selected_hull_ids.get(&rm.id) {
            None => {
                match self.hull_mesh.get(&rm.id) {
                    None => {}
                    Some(mesh) => {
                        self.change_hull_material(SELECTION_HULL_MAT, mesh.0, mesh.1);
                        self.selected_hull_ids.insert(rm.id);
                    }
                }
            }
            Some(id) => {
                self.unselect_hull_by_id(id.clone());
            }
        }
    }
    pub fn hide_hull_by_ids(&mut self, ids: HashSet<i32>) {
        warn!("unselect_hull_all");
        self.unhide_hull_all();
        ids.iter().for_each(|id| {
            match self.hull_mesh.get(id) {
                None => {}
                Some(mesh) => {
                    self.change_hull_material(HIDDEN_HULL_MAT, mesh.0, mesh.1);
                    self.hidden_hull_ids.insert(id.clone());
                }
            }
        });
    }
    pub fn hide_hull_by_part(&mut self, rm: RawMesh) {
        match self.hidden_hull_ids.get(&rm.id) {
            None => {
                match self.hull_mesh.get(&rm.id) {
                    None => {}
                    Some(mesh) => {
                        self.change_hull_material(HIDDEN_HULL_MAT, mesh.0, mesh.1);
                        self.hidden_hull_ids.insert(rm.id);
                    }
                }
            }
            Some(id) => {
                //self.unselect_hull_by_id(id.clone());
            }
        }
    }
    pub fn hide_hull_by_id(&mut self, oid: i32) {
        match self.hidden_hull_ids.get(&oid) {
            None => {
                match self.hull_mesh.get(&oid) {
                    None => {}
                    Some(mesh) => {
                        self.change_hull_material(HIDDEN_HULL_MAT, mesh.0, mesh.1);
                        self.hidden_hull_ids.insert(oid);
                        self.selected_hull_ids.remove(&oid);
                    }
                }
            }
            Some(id) => {
                //self.unselect_hull_by_id(id.clone());
            }
        }
    }

    fn get_default_material_by_id(&self,id:i32)->i32{
        match self.hull_mesh.get(&id) {
            None => {0}
            Some(m) => {
                let start_index=m.1;
                let mesh_v=self.hull_v[start_index as usize];
                let default_material = Material::type_to_color(mesh_v.material_index);
                default_material
            }
        }

    }
    pub fn unselect_hull_by_id(&mut self, id: i32) {
        match self.hull_mesh.get(&id) {
            None => {}
            Some(m) => {
                let start_index=m.1;
                let mesh_v=self.hull_v[start_index as usize];
                let default_material = Material::type_to_color(mesh_v.material_index);
                self.change_hull_material(default_material, m.0, m.1);
                self.selected_hull_ids.remove(&id);
            }
        }
    }
    pub fn unselect_hull_all(&mut self) {
        let selected: Vec<i32> = self.selected_hull_ids.clone().into_iter().collect();
        selected.iter().for_each(|id| {
            match self.hull_mesh.get(&id) {
                None => {}
                Some(m) => {
                    let default_material =self.get_default_material_by_id(id.clone());
                    self.change_hull_material(default_material, m.0, m.1);
                    self.selected_hull_ids.remove(&id);
                }
            }
        });
    }
    pub fn unhide_hull_all(&mut self) {
        let hiddens: Vec<i32> = self.hidden_hull_ids.clone().into_iter().collect();
        hiddens.iter().for_each(|id| {
            match self.hull_mesh.get(&id) {
                None => {}
                Some(m) => {
                    let default_material =self.get_default_material_by_id(id.clone());
                    self.change_hull_material(default_material, m.0, m.1);
                    self.hidden_hull_ids.remove(&id);
                }
            }
        });
    }
    pub fn unhide_hull_by_id(&mut self, id: i32) {
        match self.hull_mesh.get(&id) {
            None => {}
            Some(m) => {
                let default_material =self.get_default_material_by_id(id.clone());
                self.change_hull_material(default_material, m.0, m.1);
                self.hidden_hull_ids.remove(&id);
            }
        }
    }
    fn change_hull_material(&mut self, mat_indx: i32, start: i32, end: i32) {
        self.hull_metadata[start as usize..=end as usize].iter_mut().for_each(|m| {
            *m = mat_indx;
        });
        self.is_hull_metadata_dirty = true;
    }
    fn resize_hull_buffers(&mut self) {
        self.hull_index_buffer = self.device.read().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer Hull"),
            contents: bytemuck::cast_slice(&self.hull_i),
            usage: wgpu::BufferUsages::INDEX,
        });
        self.hull_vertex_buffer = self.device.read().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer Hull"),
            contents: bytemuck::cast_slice(&self.hull_v),
            usage: wgpu::BufferUsages::VERTEX,
        });
    }
    /*fn convert_hull_parts_to_buffers(&mut self) -> (Vec<MeshVertex>, Vec<i32>, Vec<i32>) {
        let mut indicies: Vec<i32> = vec![];
        let mut meta_data: Vec<i32> = vec![];
        let mut vertexes: Vec<MeshVertex> = vec![];

        let mut ind_offset = 0;
        let mut hashindex = 0;
        self.hull_mesh.iter_mut().for_each(|hashed_mesh| {
            let m = &hashed_mesh.1.2;
            hashed_mesh.1.0 = hashindex;
            m.vertex_normal.chunks(6).for_each(|vn| {
                let v: MeshVertex = MeshVertex::new(vn[0], vn[1], vn[2], vn[3], vn[4], vn[5], m.color_indx, m.id);
                vertexes.push(v);
                hashindex = hashindex + 1;
                meta_data.push(m.color_indx);
            });
            hashed_mesh.1.1 = hashindex - 1;
            m.indx.iter().for_each(|indx| {
                indicies.push(*indx + ind_offset);
            });
            ind_offset = ind_offset + m.indx.len() as i32;
        });
        (vertexes, indicies, meta_data)
    }*/
    fn refresh_hull_remote_selected(&mut self) {
        #[cfg(target_arch = "wasm32")]
        hull_state::select_hull_parts_remote(web_sys::js_sys::Int32Array::from(Vec::from_iter(self.selected_hull_ids.clone()).as_slice()));
    }
    pub fn zoom_to(&mut self, oid: i32) {
        match self.hull_mesh.get(&oid) {
            None => {}
            Some(p) => {
                let start_indx=p.0;
                let bbx_indx = p.2;
                let bbx =&self.bbxs[bbx_indx as usize];
                let mesh=self.hull_v[start_indx as usize];
                let p: Point3<f32> = Point3::new(mesh.normal[0], mesh.normal[1], mesh.normal[2]);
                let offset = 10 as f32;
                let eye = p - SHIP_FORWARD * offset;
                self.camera.move_and_look_at(eye, p);
            }
        }
    }
    fn refresh_hull_remote_hidden(&mut self) {
        #[cfg(target_arch = "wasm32")]
        hull_state::hide_hull_parts_remote(web_sys::js_sys::Int32Array::from(Vec::from_iter(self.hidden_hull_ids.clone()).as_slice()));
    }
}