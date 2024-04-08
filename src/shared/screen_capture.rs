use std::ops::Sub;
use std::rc::Rc;
use std::str::FromStr;
use cgmath::{InnerSpace, Point3, Vector3};
use cgmath::num_traits::Float;


use parking_lot::RwLock;
use tokio::sync::mpsc::{Receiver, Sender};


use wgpu::{Buffer, BufferSlice, BufferView, Device};
use crate::device::message_controller::{MessageController, SnapMode};
use crate::scene::gpu_mem::{unpack_id, unpack_packid};


use crate::shared::Triangle;


pub struct ScreenCapture {
    window_width: usize,
    window_hight: usize,
    image_width: usize,
    raw_image: Vec<i32>,
    sel_output_buffer: Rc<RwLock<Buffer>>,
    is_captured: bool,
    is_map_requested: bool,
    sender: Sender<bool>,
    receiver: Receiver<bool>,
    counter: u32,
}

impl ScreenCapture {
    pub fn new(device: Rc<RwLock<Device>>) -> Self {
        let (tx, rx): (Sender<bool>, Receiver<bool>) = tokio::sync::mpsc::channel(16);
        let sel_output_buffer_desc = wgpu::BufferDescriptor {
            size: (100 * 100 * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let sel_output_buffer: Buffer = device.read().create_buffer(&sel_output_buffer_desc);
        Self {
            window_width: 21,
            window_hight: 21,
            image_width: 21,
            raw_image: vec![],
            sel_output_buffer: Rc::new(RwLock::new(sel_output_buffer)),
            is_captured: false,
            is_map_requested: false,
            sender: tx,
            receiver: rx,
            counter: 0,
        }
    }

    //this buff has been created because mapsync doesn't work in webassembly (can't await). So it takes 3 steps
// 1. capture buffer by get_capture_buffer a
// 2. next render step map async
// 3. thanks to tokio channels, next render steps just check message from async. it means that it has beenmapped properly
// 4. read values, unmap buffer, reset all flags
    pub fn refresh(&mut self, mc: Rc<RwLock<MessageController>>) {
        match self.receiver.try_recv() {
            Ok(_) => {
                let mut result: Vec<i32> = vec![];
                let handler = self.sel_output_buffer.clone();
                {
                    let pointer = handler.read();
                    let buffer_slice: BufferSlice = pointer.slice(..);
                    let data: BufferView = buffer_slice.get_mapped_range();
                    result.extend_from_slice(bytemuck::cast_slice(&data));
                }
                handler.read().unmap();
                self.is_captured = false;
                self.is_map_requested = false;
                println!("TEXELS_READY WITH SIZE= {} ", result.len());
                self.raw_image = result;
                // DO OFFSCREEN ANALISYS
                self.is_captured = false;
                self.is_map_requested = false;
            }
            Err(_e) => {}
        }
        let im_w: i32 = self.image_width as i32;
        let im_h = self.window_hight as i32;
        let x: i32 = {
            let mx = mc.read().get_mouse_pos().x as i32;
            if mx < 0 {
                0
            } else {
                mx
            }
        };
        let y: i32 = {
            let my = mc.read().get_mouse_pos().y as i32;
            if my < 0 {
                0
            } else {
                my
            }
        };
        if self.raw_image.len() > 0 && !mc.read().is_capture_screen_requested {
            if (im_w - x) > 10 && x > 10 && (im_h - y) > 10 && y > 10 {
                let indx = (im_w * y * 4 + x * 4) as usize;
                let x0 = self.raw_image[indx];
                let y0 = self.raw_image[indx + 1];
                let z0 = self.raw_image[indx + 2];
                let id0 = self.raw_image[indx + 3].clone();



                mc.write().active_id = unpack_id(id0 as u32);
                mc.write().set_pack_id(unpack_packid(id0 as u32));
                mc.write().active_point = Point3::new(x0 as f32/1000.0, y0 as f32/1000.0, z0 as f32/1000.0);
                //println!("PACK_ID Fl={} {}", unpack_id(id0 as u32), unpack_packid(id0 as u32));

                //IF SNAP ENABLED
                if mc.read().snap_mode != SnapMode::Disabled {
                    let mut bricked: Vec<Vec<PixelData>> = vec![];
                    for row in y - 10..y + 11 {
                        let mut bricked_row: Vec<PixelData> = vec![];
                        for col in x - 10..x + 11 {
                            let indx = (im_w * row * 4 + col * 4) as usize;
                            let x0 = self.raw_image[indx];
                            let y0 = self.raw_image[indx + 1];
                            let z0 = self.raw_image[indx + 2];
                            let id0 = self.raw_image[indx + 3] as u32;

                            bricked_row.push(
                                PixelData {
                                    id: unpack_id(id0),
                                    pack_id: unpack_packid(id0),
                                    point_on_tri: Point3::new(x0 as f32 /1000.0, y0 as f32/1000.0, z0 as f32/1000.0),
                                }
                            );
                        }
                        bricked.push(bricked_row);
                    }
                    let (snap_shader_index, snap_vrtx, snap_vrtx_dist, edge_vrtx, edge_vrtx_dist,pack_id) = self.analyze_texels(mc.clone(), bricked);
                    match snap_vrtx {
                        None => {
                            mc.write().active_point = Point3::new(f32::max_value(), f32::max_value(), f32::max_value());
                        }
                        Some(snap_vrtx) => {
                            match edge_vrtx {
                                None => {
                                    mc.write().active_id = snap_shader_index as u32;
                                    //mc.write().set_pack_id(pack_id);
                                    mc.write().active_point = snap_vrtx;
                                    //println!("ACTIVE ID {}",snap_mesh_id);
                                }
                                Some(_edge) => {
                                    mc.write().active_id = snap_shader_index as u32;
                                    //mc.write().set_pack_id(pack_id);
                                    //println!("ACTIVE ID {}",snap_mesh_id);
                                    if edge_vrtx_dist < snap_vrtx_dist {
                                        //mc.write().active_point = edge;
                                        mc.write().active_point = snap_vrtx;
                                    } else {
                                        mc.write().active_point = snap_vrtx;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            mc.write().active_id = 0;
            mc.write().active_point = Point3::new(f32::max_value(), f32::max_value(), f32::max_value());
            self.raw_image = vec![];
        }
    }

    pub fn copy_to_host(&mut self) {
        let handler = self.sel_output_buffer.clone();
        let pointer = handler.read();
        let buffer_slice: BufferSlice = pointer.slice(..);
        let sender = self.sender.clone();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            match result {
                Ok(_) => {
                    println!("SEND ");
                    sender.try_send(true);
                }
                Err(_) => {}
            }
        });
    }

    pub fn get_capture_buffer(&mut self, device: Rc<RwLock<Device>>, window_width: usize, window_hight: usize, image_width: usize) -> Rc<RwLock<Buffer>> {
        self.is_captured = true;
        if self.window_width != window_width || self.window_hight != window_hight {
            self.window_width = window_width;
            self.window_hight = window_hight;
            self.image_width = image_width;

            let sel_output_buffer_desc = wgpu::BufferDescriptor {
                size: (image_width * window_hight * 4 * 4) as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                label: None,
                mapped_at_creation: false,
            };
            let sel_output_buffer: Buffer = device.read().create_buffer(&sel_output_buffer_desc);
            self.sel_output_buffer = Rc::new(RwLock::new(sel_output_buffer));
            self.sel_output_buffer.clone()
        } else {
            self.sel_output_buffer.clone()
        }
    }

    pub fn is_captured(&self) -> bool { self.is_captured }

    fn analyze_texels(&mut self, mc: Rc<RwLock<MessageController>>, bricked: Vec<Vec<PixelData>>) -> (i32, Option<Point3<f32>>, f32, Option<Point3<f32>>, f32, u32) {
        let mouse_wpos = mc.read().scene_state.camera.mouse_wpos;
        let mouse_world_ray = mc.read().scene_state.camera.mouse_wray.normalize();

        let mut snap_shader_index: i32 = 0;
        let mut snap_vrtx: Option<Point3<f32>> = None;
        let mut snap_vrtx_dist: f32 = f32::max_value();
        let mut edge_vrtx: Option<Point3<f32>> = None;
        let mut edge_vrtx_dist: f32 = f32::max_value();
        let mut pack_id: u32 = u32::max_value();
        let mut total = 0;
        for row in (1..21).step_by(3) {
            for col in (1..21).step_by(3) {
                let p00 = &bricked[row - 1][col - 1];
                let p01 = &bricked[row - 1][col];
                let p02 = &bricked[row - 1][col + 1];

                let p10 = &bricked[row][col - 1];
                let p12 = &bricked[row][col + 1];

                let p20 = &bricked[row + 1][col - 1];
                let p21 = &bricked[row + 1][col];
                let p22 = &bricked[row + 1][col + 1];

                let cp = &bricked[row][col];
                //println!("CP {:?}",p00.id);


                match mc.read().scene_state.get_triangle_by_index(cp.id as usize, cp.pack_id as usize) {
                    //CHECK OUTSIDE AREA
                    None => {
                        match mc.read().scene_state.get_triangle_by_index(p00.id as usize, p00.pack_id as usize) {
                            None => {}
                            Some((_meshid, loc_tri)) => {
                                let mouse_world_pos = mouse_wpos + mouse_world_ray * mouse_wpos.sub(p00.point_on_tri).magnitude();
                                let (vrtx, vrtx_dist, edge, edge_dist) = find_nearest(mouse_world_pos, loc_tri);
                                if vrtx_dist < snap_vrtx_dist {
                                    snap_vrtx = Some(vrtx);
                                    snap_vrtx_dist = vrtx_dist;
                                    snap_shader_index = p00.id as i32;
                                }
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                                pack_id= p00.pack_id;
                            }
                        }
                        match mc.read().scene_state.get_triangle_by_index(p01.id as usize, p01.pack_id as usize) {
                            None => {}
                            Some((_meshid, loc_tri)) => {
                                let mouse_world_pos = mouse_wpos + mouse_world_ray * mouse_wpos.sub(p01.point_on_tri).magnitude();
                                let (vrtx, vrtx_dist, edge, edge_dist) = find_nearest(mouse_world_pos, loc_tri);
                                if vrtx_dist < snap_vrtx_dist {
                                    snap_vrtx = Some(vrtx);
                                    snap_vrtx_dist = vrtx_dist;
                                    snap_shader_index = p01.id as i32;
                                }
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                                pack_id= p01.pack_id;
                            }
                        }
                        match mc.read().scene_state.get_triangle_by_index(p02.id as usize, p02.pack_id as usize) {
                            None => {}
                            Some((_meshid, loc_tri)) => {
                                let mouse_world_pos = mouse_wpos + mouse_world_ray * mouse_wpos.sub(p02.point_on_tri).magnitude();
                                let (vrtx, vrtx_dist, edge, edge_dist) = find_nearest(mouse_world_pos, loc_tri);
                                if vrtx_dist < snap_vrtx_dist {
                                    snap_vrtx = Some(vrtx);
                                    snap_vrtx_dist = vrtx_dist;
                                    snap_shader_index = p02.id as i32;
                                }
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                                pack_id= p02.pack_id;
                            }
                        }
                        match mc.read().scene_state.get_triangle_by_index(p10.id as usize, p10.pack_id as usize) {
                            None => {}
                            Some((_meshid, loc_tri)) => {
                                let mouse_world_pos = mouse_wpos + mouse_world_ray * mouse_wpos.sub(p10.point_on_tri).magnitude();
                                let (vrtx, vrtx_dist, edge, edge_dist) = find_nearest(mouse_world_pos, loc_tri);
                                if vrtx_dist < snap_vrtx_dist {
                                    snap_vrtx = Some(vrtx);
                                    snap_vrtx_dist = vrtx_dist;
                                    snap_shader_index = p10.id as i32;
                                }
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                                pack_id= p10.pack_id;
                            }
                        }
                        match mc.read().scene_state.get_triangle_by_index(p12.id as usize, p12.pack_id as usize) {
                            None => {}
                            Some((_meshid, loc_tri)) => {
                                let mouse_world_pos = mouse_wpos + mouse_world_ray * mouse_wpos.sub(p12.point_on_tri).magnitude();
                                let (vrtx, vrtx_dist, edge, edge_dist) = find_nearest(mouse_world_pos, loc_tri);
                                if vrtx_dist < snap_vrtx_dist {
                                    snap_vrtx = Some(vrtx);
                                    snap_vrtx_dist = vrtx_dist;
                                    snap_shader_index = p12.id as i32;
                                }
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                                pack_id= p12.pack_id;
                            }
                        }
                        match mc.read().scene_state.get_triangle_by_index(p20.id as usize, p20.pack_id as usize) {
                            None => {}
                            Some((_meshid, loc_tri)) => {
                                let mouse_world_pos = mouse_wpos + mouse_world_ray * mouse_wpos.sub(p20.point_on_tri).magnitude();
                                let (vrtx, vrtx_dist, edge, edge_dist) = find_nearest(mouse_world_pos, loc_tri);
                                if vrtx_dist < snap_vrtx_dist {
                                    snap_vrtx = Some(vrtx);
                                    snap_vrtx_dist = vrtx_dist;
                                    snap_shader_index = p20.id as i32;
                                }
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                                pack_id= p20.pack_id;
                            }
                        }
                        match mc.read().scene_state.get_triangle_by_index(p21.id as usize, p21.pack_id as usize) {
                            None => {}
                            Some((_meshid, loc_tri)) => {
                                let mouse_world_pos = mouse_wpos + mouse_world_ray * mouse_wpos.sub(p21.point_on_tri).magnitude();
                                let (vrtx, vrtx_dist, edge, edge_dist) = find_nearest(mouse_world_pos, loc_tri);
                                if vrtx_dist < snap_vrtx_dist {
                                    snap_vrtx = Some(vrtx);
                                    snap_vrtx_dist = vrtx_dist;
                                    snap_shader_index = p21.id as i32;
                                }
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                                pack_id= p21.pack_id;
                            }
                        }
                        match mc.read().scene_state.get_triangle_by_index(p22.id as usize, p22.pack_id as usize) {
                            None => {}
                            Some((_meshid, loc_tri)) => {
                                let mouse_world_pos = mouse_wpos + mouse_world_ray * mouse_wpos.sub(p22.point_on_tri).magnitude();
                                let (vrtx, vrtx_dist, edge, edge_dist) = find_nearest(mouse_world_pos, loc_tri);
                                if vrtx_dist < snap_vrtx_dist {
                                    snap_vrtx = Some(vrtx);
                                    snap_vrtx_dist = vrtx_dist;
                                    snap_shader_index = p22.id as i32;
                                }
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                                pack_id= p22.pack_id;
                            }
                        }
                    }
                    //CHECK INSIDE AREA
                    Some(base_tri) => {
                        let base_tri_normal: Vector3<f32> = base_tri.1.normal;
                        let ray_center_pos = mouse_wpos + mouse_world_ray * mouse_wpos.sub(cp.point_on_tri).magnitude();
                        let (vrtx, vrtx_dist, edge, edge_dist) = find_nearest(ray_center_pos, base_tri.1);
                        match mc.read().scene_state.get_triangle_by_index(p00.id as usize, p00.pack_id as usize) {
                            None => {
                                //it means we are near for border this pixel are outside of triangle
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                                pack_id= p00.pack_id;
                            }
                            Some((_meshid, loc_tri)) => {
                                //it means we are near for other triangle we need to check normals to disable snap flat parts of face
                                let loc_tri_normal: Vector3<f32> = loc_tri.normal;
                                let cos_f: f32 = loc_tri_normal.dot(base_tri_normal).abs();

                                //if true angle less 30 degree we should discard result
                                if cos_f > 0.86602540378 {} else {
                                    if vrtx_dist < snap_vrtx_dist {
                                        snap_vrtx = Some(vrtx);
                                        snap_vrtx_dist = vrtx_dist;
                                        snap_shader_index = p00.id as i32;
                                    }
                                    if edge_dist < edge_vrtx_dist {
                                        edge_vrtx = edge;
                                        edge_vrtx_dist = edge_dist;
                                    }
                                }
                            }
                        }
                        match mc.read().scene_state.get_triangle_by_index(p01.id as usize, p01.pack_id as usize) {
                            None => {
                                //it means we are near for border this pixel are outside of triangle
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                            }
                            Some((_meshid, loc_tri)) => {
                                //it means we are near for other triangle we need to check normals to disable snap flat parts of face
                                let loc_tri_normal: Vector3<f32> = loc_tri.normal;
                                let cos_f: f32 = loc_tri_normal.dot(base_tri_normal).abs();

                                //if true angle less 30 degree we should discard result
                                if cos_f > 0.86602540378 {} else {
                                    if vrtx_dist < snap_vrtx_dist {
                                        snap_vrtx = Some(vrtx);
                                        snap_vrtx_dist = vrtx_dist;
                                        snap_shader_index = p00.id as i32;
                                    }
                                    if edge_dist < edge_vrtx_dist {
                                        edge_vrtx = edge;
                                        edge_vrtx_dist = edge_dist;
                                    }
                                }
                            }
                        }
                        match mc.read().scene_state.get_triangle_by_index(p02.id as usize, p02.pack_id as usize) {
                            None => {
                                //it means we are near for border this pixel are outside of triangle
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                            }
                            Some((_meshid, loc_tri)) => {
                                //it means we are near for other triangle we need to check normals to disable snap flat parts of face
                                let loc_tri_normal: Vector3<f32> = loc_tri.normal;
                                let cos_f: f32 = loc_tri_normal.dot(base_tri_normal).abs();

                                //if true angle less 30 degree we should discard result
                                if cos_f > 0.86602540378 {} else {
                                    if vrtx_dist < snap_vrtx_dist {
                                        snap_vrtx = Some(vrtx);
                                        snap_vrtx_dist = vrtx_dist;
                                        snap_shader_index = p00.id as i32;
                                    }
                                    if edge_dist < edge_vrtx_dist {
                                        edge_vrtx = edge;
                                        edge_vrtx_dist = edge_dist;
                                    }
                                }
                            }
                        }
                        match mc.read().scene_state.get_triangle_by_index(p10.id as usize, p10.pack_id as usize) {
                            None => {
                                //it means we are near for border this pixel are outside of triangle
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                            }
                            Some((_meshid, loc_tri)) => {
                                //it means we are near for other triangle we need to check normals to disable snap flat parts of face
                                let loc_tri_normal: Vector3<f32> = loc_tri.normal;
                                let cos_f: f32 = loc_tri_normal.dot(base_tri_normal).abs();

                                //if true angle less 30 degree we should discard result
                                if cos_f > 0.86602540378 {} else {
                                    if vrtx_dist < snap_vrtx_dist {
                                        snap_vrtx = Some(vrtx);
                                        snap_vrtx_dist = vrtx_dist;
                                        snap_shader_index = p00.id as i32;
                                    }
                                    if edge_dist < edge_vrtx_dist {
                                        edge_vrtx = edge;
                                        edge_vrtx_dist = edge_dist;
                                    }
                                }
                            }
                        }
                        match mc.read().scene_state.get_triangle_by_index(p12.id as usize, p12.pack_id as usize) {
                            None => {
                                //it means we are near for border this pixel are outside of triangle
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                            }
                            Some((_meshid, loc_tri)) => {
                                //it means we are near for other triangle we need to check normals to disable snap flat parts of face
                                let loc_tri_normal: Vector3<f32> = loc_tri.normal;
                                let cos_f: f32 = loc_tri_normal.dot(base_tri_normal).abs();

                                //if true angle less 30 degree we should discard result
                                if cos_f > 0.86602540378 {} else {
                                    if vrtx_dist < snap_vrtx_dist {
                                        snap_vrtx = Some(vrtx);
                                        snap_vrtx_dist = vrtx_dist;
                                        snap_shader_index = p00.id as i32;
                                    }
                                    if edge_dist < edge_vrtx_dist {
                                        edge_vrtx = edge;
                                        edge_vrtx_dist = edge_dist;
                                    }
                                }
                            }
                        }
                        match mc.read().scene_state.get_triangle_by_index(p20.id as usize, p20.pack_id as usize) {
                            None => {
                                //it means we are near for border this pixel are outside of triangle
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                            }
                            Some((_meshid, loc_tri)) => {
                                //it means we are near for other triangle we need to check normals to disable snap flat parts of face
                                let loc_tri_normal: Vector3<f32> = loc_tri.normal;
                                let cos_f: f32 = loc_tri_normal.dot(base_tri_normal).abs();

                                //if true angle less 30 degree we should discard result
                                if cos_f > 0.86602540378 {} else {
                                    if vrtx_dist < snap_vrtx_dist {
                                        snap_vrtx = Some(vrtx);
                                        snap_vrtx_dist = vrtx_dist;
                                        snap_shader_index = p00.id as i32;
                                    }
                                    if edge_dist < edge_vrtx_dist {
                                        edge_vrtx = edge;
                                        edge_vrtx_dist = edge_dist;
                                    }
                                }
                            }
                        }
                        match mc.read().scene_state.get_triangle_by_index(p21.id as usize, p21.pack_id as usize) {
                            None => {
                                //it means we are near for border this pixel are outside of triangle
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                            }
                            Some((_meshid, loc_tri)) => {
                                //it means we are near for other triangle we need to check normals to disable snap flat parts of face
                                let loc_tri_normal: Vector3<f32> = loc_tri.normal;
                                let cos_f: f32 = loc_tri_normal.dot(base_tri_normal).abs();

                                //if true angle less 30 degree we should discard result
                                if cos_f > 0.86602540378 {} else {
                                    if vrtx_dist < snap_vrtx_dist {
                                        snap_vrtx = Some(vrtx);
                                        snap_vrtx_dist = vrtx_dist;
                                        snap_shader_index = p00.id as i32;
                                    }
                                    if edge_dist < edge_vrtx_dist {
                                        edge_vrtx = edge;
                                        edge_vrtx_dist = edge_dist;
                                    }
                                }
                            }
                        }
                        match mc.read().scene_state.get_triangle_by_index(p22.id as usize, p22.pack_id as usize) {
                            None => {
                                //it means we are near for border this pixel are outside of triangle
                                if edge_dist < edge_vrtx_dist {
                                    edge_vrtx = edge;
                                    edge_vrtx_dist = edge_dist;
                                }
                            }
                            Some((_meshid, loc_tri)) => {
                                //it means we are near for other triangle we need to check normals to disable snap flat parts of face
                                let loc_tri_normal: Vector3<f32> = loc_tri.normal;
                                let cos_f: f32 = loc_tri_normal.dot(base_tri_normal).abs();

                                //if true angle less 30 degree we should discard result
                                if cos_f > 0.86602540378 {} else {
                                    if vrtx_dist < snap_vrtx_dist {
                                        snap_vrtx = Some(vrtx);
                                        snap_vrtx_dist = vrtx_dist;
                                        snap_shader_index = p00.id as i32;
                                    }
                                    if edge_dist < edge_vrtx_dist {
                                        edge_vrtx = edge;
                                        edge_vrtx_dist = edge_dist;
                                    }
                                }
                            }
                        }
                    }
                };
                total = total + 1;
            }
        }


        (snap_shader_index, snap_vrtx, snap_vrtx_dist, edge_vrtx, edge_vrtx_dist,pack_id)
    }
}

fn find_nearest(p: Point3<f32>, tri: Triangle) -> (Point3<f32>, f32, Option<Point3<f32>>, f32) {
    let mut out_proj_dist: f32 = f32::max_value();
    let mut out_proj: Option<Point3<f32>> = None;


    let a: Point3<f32> = tri.p[0];
    let b: Point3<f32> = tri.p[1];
    let c: Point3<f32> = tri.p[2];
    let (proj_ab, d_ab, cos_ab) = project_point_to_line(p, a, b);
    let (proj_bc, d_bc, cos_bc) = project_point_to_line(p, b, c);
    let (proj_ca, d_ca, cos_ca) = project_point_to_line(p, c, a);
    if cos_ab < 0.0 && d_ab < out_proj_dist {
        out_proj_dist = d_ab;
        out_proj = Some(proj_ab)
    }
    if cos_bc < 0.0 && d_bc < out_proj_dist {
        out_proj_dist = d_bc;
        out_proj = Some(proj_bc)
    }
    if cos_ca < 0.0 && d_ca < out_proj_dist {
        out_proj_dist = d_ca;
        out_proj = Some(proj_ca)
    }

    let mut out_vrtx_dist: f32 = f32::max_value();
    let mut out_vrtx: Point3<f32> = a;

    let p_a = p.sub(a).magnitude();
    let p_b = p.sub(b).magnitude();
    let p_c = p.sub(c).magnitude();
    if p_a < out_vrtx_dist {
        out_vrtx = a;
        out_vrtx_dist = p_a;
    }
    if p_b < out_vrtx_dist {
        out_vrtx = b;
        out_vrtx_dist = p_b;
    }
    if p_c < out_vrtx_dist {
        out_vrtx = c;
        out_vrtx_dist = p_c;
    }

    //println!("{} {}",out_vrtx_dist,out_proj_dist);
    (out_vrtx, out_vrtx_dist, out_proj, out_proj_dist)
}

fn project_point_to_line(p: Point3<f32>, a: Point3<f32>, b: Point3<f32>) -> (Point3<f32>, f32, f32) {
    let line_v = b.sub(a);
    let p_v = p.sub(a);
    let ap = line_v * line_v.dot(p_v) / line_v.dot(line_v);
    let proj_poin = a + ap;
    let d = p.sub(proj_poin).magnitude();
    let a_p = proj_poin.sub(a);
    let b_p = proj_poin.sub(b);
    let cos_phi = a_p.dot(b_p);
    //println!("EDGE D {}",d);
    //VALID ONLY cos_phi<0
    (proj_poin, d, cos_phi)
}

#[derive(Clone)]
struct PixelData {
    id: u32,
    pack_id: u32,
    point_on_tri: Point3<f32>,
}

struct PixelDataCandidate {
    pixel: PixelData,
    triangle: Triangle,
}