use std::collections::{HashMap, HashSet};


use std::rc::Rc;
use cgmath::{InnerSpace, Point3};

use itertools::Itertools;
use log::{warn};
use parking_lot::RwLock;


use truck_base::bounding_box::{BoundingBox};
use truck_polymesh::Attributes;

use wgpu::{Buffer, Device};
use crate::device::message_controller::{ActionType};
use crate::scene::{mesh_loader, RawMesh};
use crate::shared::mesh_common::MeshVertex;
use wgpu::util::DeviceExt;
use crate::gui::camera_base::{CameraBase, SHIP_FORWARD};
use crate::gui::slicer::Slicer;
use crate::remote::hull_state;
use crate::scene::gpu_mem::{GpuMem, unpack_id, unpack_packid};
use crate::scene::mesh_loader::read_hull_unpacked_new_format;
use crate::shared::materials_lib::{HIDDEN_HULL_MAT, Material, SELECTION_HULL_MAT};
use crate::shared::Triangle;

#[cfg(target_arch = "wasm32")]
use crate::remote::hull_state::{get_bbx_array, get_index_array, get_types_array, get_vertex_array};
use crate::remote::hull_state::get_mesh_vertex_by_id;

pub struct SceneState {
    device: Rc<RwLock<Device>>,
    pub gpu_mems: Vec<GpuMem>,
    pub tot_bbx: BoundingBox<Point3<f64>>,
    pub camera: CameraBase,
    pub slicer: Slicer,
    selected_hull_ids: HashSet<i32>,
    hidden_hull_ids: HashSet<i32>,
    pub is_snap_dirty: bool,
    pub snap_vertex_buffer: Buffer,

}

impl SceneState {
    pub fn new(device: Rc<RwLock<Device>>) -> Self {
        let mut gpu_mems: Vec<GpuMem> = vec![];
        gpu_mems.push(GpuMem::new(device.clone(), 0));
        gpu_mems.push(GpuMem::new(device.clone(), 1));
        gpu_mems.push(GpuMem::new(device.clone(), 2));
        gpu_mems.push(GpuMem::new(device.clone(), 3));
        gpu_mems.push(GpuMem::new(device.clone(), 4));
        gpu_mems.push(GpuMem::new(device.clone(), 5));
        gpu_mems.push(GpuMem::new(device.clone(), 6));
        gpu_mems.push(GpuMem::new(device.clone(), 7));

        let snap_vertex_buffer: Buffer = device.clone().read().create_buffer(&wgpu::BufferDescriptor {
            label: Some("Snap Vertex"),
            size: 128,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            device: device,
            gpu_mems: gpu_mems,
            tot_bbx: Default::default(),
            camera: CameraBase::default(),
            slicer: Slicer::default(),
            selected_hull_ids: HashSet::new(),
            hidden_hull_ids: HashSet::new(),
            is_snap_dirty: false,
            snap_vertex_buffer: snap_vertex_buffer,
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn set_hull_mesh0(&mut self) {
        {
            let vu = get_vertex_array(0);
            let iu = get_index_array(0);
            let bu = get_bbx_array(0);
            let tu = get_types_array(0);
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) = read_hull_unpacked_new_format(vu.to_vec(), iu.to_vec(), bu.to_vec(), tu.to_vec());
            let package_id: u32 = 0;
            warn!("LOADED {} {:?}",package_id, out_bbx.diagonal().magnitude());
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 0");
    }
    #[cfg(target_arch = "wasm32")]
    pub fn set_hull_mesh1(&mut self) {
        {
            let vu = get_vertex_array(1);
            let iu = get_index_array(1);
            let bu = get_bbx_array(1);
            let tu = get_types_array(1);
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) = read_hull_unpacked_new_format(vu.to_vec(), iu.to_vec(), bu.to_vec(), tu.to_vec());
            warn!("LOADED");
            let package_id: u32 = 1;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 1");
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_hull_mesh2(&mut self) {
        {
            let vu = get_vertex_array(2);
            let iu = get_index_array(2);
            let bu = get_bbx_array(2);
            let tu = get_types_array(2);
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) = read_hull_unpacked_new_format(vu.to_vec(), iu.to_vec(), bu.to_vec(), tu.to_vec());
            warn!("LOADED");
            let package_id: u32 = 2;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 2");
    }
    #[cfg(target_arch = "wasm32")]
    pub fn set_hull_mesh3(&mut self) {
        {
            let vu = get_vertex_array(3);
            let iu = get_index_array(3);
            let bu = get_bbx_array(3);
            let tu = get_types_array(3);
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) = read_hull_unpacked_new_format(vu.to_vec(), iu.to_vec(), bu.to_vec(), tu.to_vec());
            warn!("LOADED");
            let package_id: u32 = 3;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 3");
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_hull_mesh4(&mut self) {
        {
            let vu = get_vertex_array(4);
            let iu = get_index_array(4);
            let bu = get_bbx_array(4);
            let tu = get_types_array(4);
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) = read_hull_unpacked_new_format(vu.to_vec(), iu.to_vec(), bu.to_vec(), tu.to_vec());
            warn!("LOADED");
            let package_id: u32 = 4;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 4");
    }

    #[cfg(target_arch = "wasm32")]
    pub fn set_hull_mesh5(&mut self) {
        {
            let vu = get_vertex_array(5);
            let iu = get_index_array(5);
            let bu = get_bbx_array(5);
            let tu = get_types_array(5);
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) = read_hull_unpacked_new_format(vu.to_vec(), iu.to_vec(), bu.to_vec(), tu.to_vec());
            warn!("LOADED");
            let package_id: u32 = 5;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 5");
    }
    #[cfg(target_arch = "wasm32")]
    pub fn set_hull_mesh6(&mut self) {
        {
            let vu = get_vertex_array(6);
            let iu = get_index_array(6);
            let bu = get_bbx_array(6);
            let tu = get_types_array(6);
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) = read_hull_unpacked_new_format(vu.to_vec(), iu.to_vec(), bu.to_vec(), tu.to_vec());
            warn!("LOADED");
            let package_id: u32 = 6;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 6");
    }
    #[cfg(target_arch = "wasm32")]
    pub fn set_hull_mesh7(&mut self) {
        {
            let vu = get_vertex_array(7);
            let iu = get_index_array(7);
            let bu = get_bbx_array(7);
            let tu = get_types_array(7);
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) = read_hull_unpacked_new_format(vu.to_vec(), iu.to_vec(), bu.to_vec(), tu.to_vec());
            warn!("LOADED");
            let package_id: u32 = 7;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 7");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_hull_mesh0(&mut self) {
        {
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) = mesh_loader::read_hull_packed_new_format0();
            let package_id: u32 = 0;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 0");
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_hull_mesh1(&mut self) {
        {
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) = mesh_loader::read_hull_packed_new_format1();
            let package_id: u32 = 1;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 1");
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_hull_mesh2(&mut self) {
        {
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) = mesh_loader::read_hull_packed_new_format2();
            let package_id: u32 = 2;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 2");
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_hull_mesh3(&mut self) {
        {
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) = mesh_loader::read_hull_packed_new_format3();
            let package_id: u32 = 3;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 3");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_hull_mesh4(&mut self) {
        {
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) =
                mesh_loader::read_hull_packed_new_format4();
            let package_id: u32 = 4;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 4");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_hull_mesh5(&mut self) {
        {
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) =
                mesh_loader::read_hull_packed_new_format5();
            let package_id: u32 = 5;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 5");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_hull_mesh6(&mut self) {
        {
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) =
                mesh_loader::read_hull_packed_new_format6();
            let package_id: u32 = 6;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 6");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_hull_mesh7(&mut self) {
        {
            let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) =
                mesh_loader::read_hull_packed_new_format7();
            let package_id: u32 = 7;
            self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx.clone(), hull_mesh, bbxs);
            self.tot_bbx += out_bbx;
            self.slicer.set_by_bbx(&self.tot_bbx);
        }
        warn!("UNPACKED 7");
    }


    pub fn set_hull_mesh_remote(&mut self, load_level: i32, decoded_v: &[u8], decoded_i: &[u8], decoded_b: &[u8], decoded_t: &[u8]) {
        warn!("TRY LOAD HULL FROm REMOTE");
        let (hm, i, meta_data, out_bbx, hull_mesh, bbxs) =
            mesh_loader::read_hull_unpacked_new_format_remote(load_level, decoded_v, decoded_i, decoded_b, decoded_t);
        let package_id: u32 = load_level as u32;
        self.tot_bbx += &out_bbx;
        self.gpu_mems[package_id as usize].set_data(hm, i, meta_data, out_bbx, hull_mesh, bbxs);
        self.slicer.set_by_bbx(&self.tot_bbx);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn screen_oid(&mut self, action: ActionType, id: i32, pack_id: u32) -> bool {
        let mut is_scene_modified = false;
        if (id != 0) {
            let bin=get_mesh_vertex_by_id( pack_id as i32,id ).to_vec();
            let meshes: &[MeshVertex] =bytemuck::cast_slice(bin.as_slice());
            if(meshes.len()>0){
                let mesh=meshes[0];
                match action {
                    ActionType::Select => {
                        self.select_by_id(mesh.id, pack_id);
                        self.refresh_hull_remote_selected();
                        is_scene_modified
                    }
                    ActionType::Hide => {
                        self.hide_by_id(mesh.id, pack_id);
                        self.refresh_hull_remote_hidden();
                        is_scene_modified = true;
                        is_scene_modified
                    }
                    ActionType::Evaluate => {
                        is_scene_modified
                    }
                }
            }else{
                is_scene_modified
            }
        } else {
            is_scene_modified
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn screen_oid(&mut self, action: ActionType, id: i32, pack_id: u32) -> bool {
        let mut is_scene_modified = false;
        if id != 0 {
            match self.gpu_mems[pack_id as usize].v.get(id as usize) {
                None => {
                    is_scene_modified
                }
                Some(mesh) => {
                    match action {
                        ActionType::Select => {
                            self.select_by_id(mesh.id, pack_id);
                            self.refresh_hull_remote_selected();
                            is_scene_modified
                        }
                        ActionType::Hide => {
                            self.hide_by_id(mesh.id, pack_id);
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
    pub fn get_triangle_by_index(&self, index: usize, pack_id: usize) -> Option<(i32, Triangle)> {
        if index < 2 {
            None
        } else {
            self.gpu_mems[pack_id].get_triangle_by_vertex_index(index)
        }
    }
    pub fn on_render(&mut self) { self.camera.update(self.tot_bbx.clone()) }
    pub fn unselect_by_id(&mut self, id: i32) {
        self.gpu_mems.iter_mut().for_each(|mem| {
            if (mem.set_default_by_id(id.clone())) {
                self.selected_hull_ids.remove(&id);
            }
        });
    }
    pub fn unselect_all(&mut self) {
        let selected: Vec<i32> = self.selected_hull_ids.clone().into_iter().collect();
        selected.iter().for_each(|id| {
            self.gpu_mems.iter_mut().for_each(|mem| {
                if (mem.set_default_by_id(id.clone())) {
                    self.selected_hull_ids.remove(&id);
                }
            });
        });
    }
    pub fn select_by_id(&mut self, oid: i32, pack_id: u32) {
        self.unselect_all();
        if (self.gpu_mems[pack_id as usize].select_by_id(oid)) {
            self.selected_hull_ids.insert(oid);
        }
    }
    pub fn select_by_ids(&mut self, ids: HashSet<i32>) {
        self.unselect_all();
        ids.iter().for_each(|id| {
            self.gpu_mems.iter_mut().for_each(|mem| {
                if (mem.select_by_id(id.clone())) {
                    self.selected_hull_ids.insert(id.clone());
                }
            });
        });
    }
    pub fn unhide_by_id(&mut self, id: i32) {
        self.gpu_mems.iter_mut().for_each(|mem| {
            if (mem.set_default_by_id(id.clone())) {
                self.hidden_hull_ids.remove(&id);
            }
        });
    }
    pub fn unhide_all(&mut self) {
        let hidden: Vec<i32> = self.hidden_hull_ids.clone().into_iter().collect();
        hidden.iter().for_each(|id| {
            self.gpu_mems.iter_mut().for_each(|mem| {
                if (mem.set_default_by_id(id.clone())) {
                    self.hidden_hull_ids.remove(&id);
                }
            });
        });
    }
    pub fn hide_by_id(&mut self, oid: i32, pack_id: u32) {
        if (self.gpu_mems[pack_id as usize].hide_by_id(oid)) {
            self.selected_hull_ids.insert(oid);
        }
    }
    pub fn hide_by_ids(&mut self, ids: HashSet<i32>) {
        self.unhide_all();
        ids.iter().for_each(|id| {
            self.gpu_mems.iter_mut().for_each(|mem| {
                if (mem.hide_by_id(id.clone())) {
                    self.hidden_hull_ids.insert(id.clone());
                }
            });
        });
    }
    pub fn zoom_to(&mut self, oid: i32) {
        self.gpu_mems.iter().for_each(|mem| {
            match mem.get_bbx_by_oid(oid) {
                None => {}
                Some(bbx) => {
                    let center = bbx.center();
                    let p: Point3<f32> = Point3::new(center.x as f32, center.y as f32, center.z as f32);
                    let offset = 10 as f32;
                    let eye = p - SHIP_FORWARD * offset;
                    self.camera.move_and_look_at(eye, p);
                }
            }
        });
    }


    fn refresh_hull_remote_selected(&mut self) {
        //#[cfg(target_arch = "wasm32")]
        //hull_state::select_hull_parts_remote(web_sys::js_sys::Int32Array::from(Vec::from_iter(self.selected_hull_ids.clone()).as_slice()));
    }
    fn refresh_hull_remote_hidden(&mut self) {
        // #[cfg(target_arch = "wasm32")]
        //hull_state::hide_hull_parts_remote(web_sys::js_sys::Int32Array::from(Vec::from_iter(self.hidden_hull_ids.clone()).as_slice()));
    }
}