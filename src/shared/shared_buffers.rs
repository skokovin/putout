use std::mem::size_of;
use std::rc::Rc;
use cgmath::Point3;
use parking_lot::RwLock;

use wgpu::{Buffer, BufferAddress, Device, Queue};
use crate::device::message_controller::SnapMode;
use crate::shared::dimension::Dimension;
use crate::shared::materials_lib::{Material, MATERIALS_COUNT};

pub struct SharedBuffers {
    pub camera_buffer: Rc<RwLock<Buffer>>,
    pub material_buffer: Rc<RwLock<Buffer>>,
    pub light_buffer: Rc<RwLock<Buffer>>,
    pub mode_buffer: Rc<RwLock<Buffer>>,
    pub slice_buffer: Rc<RwLock<Buffer>>,
    pub snap_buffer: Rc<RwLock<Buffer>>,
    pub hull_metadata_buffer: Rc<RwLock<Buffer>>,
}

impl SharedBuffers {
    pub fn new(_device: Rc<RwLock<Device>>) -> Self {
        let device = _device.write();
        let camera_buffer: Buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: 144,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let material_buffer: Buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Material Uniform Buffer"),
            size: (size_of::<Material>() * MATERIALS_COUNT) as BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mode_buffer: Buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mode Uniform Buffer"),
            size: 16 as BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_buffer: Buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Uniform Buffer"),
            size: 48,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let slice_buffer: Buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("slice Uniform Buffer"),
            size: 32, //size: (size_of::<f32>() * 6) as BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let snap_buffer: Buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("snap Uniform Buffer"),
            size: 80,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let hull_metadata_buffer: Buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hull_metadata_buffer Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        Self {
            camera_buffer: Rc::new(RwLock::new(camera_buffer)),
            material_buffer: Rc::new(RwLock::new(material_buffer)),
            light_buffer: Rc::new(RwLock::new(light_buffer)),
            mode_buffer: Rc::new(RwLock::new(mode_buffer)),
            slice_buffer: Rc::new(RwLock::new(slice_buffer)),
            snap_buffer: Rc::new(RwLock::new(snap_buffer)),
            hull_metadata_buffer: Rc::new(RwLock::new(hull_metadata_buffer)),
        }
    }

    pub fn update_camera(&mut self, queue: Rc<RwLock<Queue>>, mvp: &[f32; 16], dirs: &[f32; 16], forward: &[f32; 3]) {
        let q = queue.write();
        let buff = self.camera_buffer.clone();
        q.write_buffer(&buff.write(), 0, bytemuck::cast_slice(mvp));
        q.write_buffer(&buff.write(), 64, bytemuck::cast_slice(dirs));
        q.write_buffer(&buff.write(), 128, bytemuck::cast_slice(forward));
    }

    pub fn update_lights(&mut self, queue: Rc<RwLock<Queue>>, light_position: &[f32; 3], eye_position: &[f32; 3], w: f32, h: f32) {
        let q = queue.write();
        let buff = self.light_buffer.clone();
        let resolution: [f32; 4] = [w, h, 0.0, 0.0];
        q.write_buffer(&buff.write(), 0, bytemuck::cast_slice(light_position));
        q.write_buffer(&buff.write(), 16, bytemuck::cast_slice(eye_position));
        q.write_buffer(&buff.write(), 32, bytemuck::cast_slice(&resolution));
    }

    pub fn update_slicer(&mut self, queue: Rc<RwLock<Queue>>, slicer_pos: &[f32; 6]) {
        let q = queue.write();
        let buff = self.slice_buffer.clone();
        q.write_buffer(&buff.write(), 0, bytemuck::cast_slice(slicer_pos));
    }

    pub fn update_material(&self, queue: Rc<RwLock<Queue>>, materials: &Vec<Material>) {
        let q = queue.write();
        let buff = self.material_buffer.clone();
        q.write_buffer(&buff.write(), 0, bytemuck::cast_slice(materials));
    }

    pub fn update_snap(&self, queue: Rc<RwLock<Queue>>, active_point: Point3<f32>, dimension: Dimension, snap_mode: SnapMode) {
        let q = queue.write();
        let buff = self.snap_buffer.clone();
        let v: [f32; 4] = [active_point.x, active_point.y, active_point.z, 1.0];

        let p0: [f32; 4] = [dimension.p0.x, dimension.p0.y, dimension.p0.z, 1.0];
        let p1: [f32; 4] = [dimension.p1.x, dimension.p1.y, dimension.p1.z, 1.0];
        let p2: [f32; 4] = [dimension.p2.x, dimension.p2.y, dimension.p2.z, 1.0];
        let mode: [i32; 4] = [snap_mode as i32, dimension.mode as i32, 0, 0];

        q.write_buffer(&buff.write(), 0, bytemuck::cast_slice(&v));

        q.write_buffer(&buff.write(), 16, bytemuck::cast_slice(&p0));
        q.write_buffer(&buff.write(), 32, bytemuck::cast_slice(&p1));
        q.write_buffer(&buff.write(), 48, bytemuck::cast_slice(&p2));
        q.write_buffer(&buff.write(), 64, bytemuck::cast_slice(&mode));
    }

    pub fn update_hull_metadata(&mut self, device: Rc<RwLock<Device>>, queue: Rc<RwLock<Queue>>, h_m_d: &Vec<i32>) {
        {
            let mut buff = self.hull_metadata_buffer.clone();
            let new_size: usize = if (h_m_d.len() == 0) { 16 } else {
                let curr_size = (h_m_d.len() * 4) as f32;
                (((curr_size / 16.0).ceil() + 1.0) * 16.0) as usize
            };
            let currsize = buff.read().size();
            if (currsize != new_size as u64) {
                let d = device.clone();
                let new_buff: Buffer = d.read().create_buffer(&wgpu::BufferDescriptor {
                    label: Some("hull_metadata_buffer Buffer"),
                    size: (new_size) as BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });
                self.hull_metadata_buffer = Rc::new(RwLock::new(new_buff));
            }
        }
        let q = queue.write();
        let mut buff = self.hull_metadata_buffer.clone();
        q.write_buffer(&buff.write(), 0, bytemuck::cast_slice(h_m_d.as_ref()));
    }
}