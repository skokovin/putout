use std::mem;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub material_index: i32,
    pub id: i32,
}

impl MeshVertex {
    pub fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 1.0],
            normal: [1.0, 0.0, 0.0, 1.0],
            material_index: 0,
            id: 0,
        }
    }
    pub fn new(vx: f32, vy: f32, vz: f32, nx: f32, ny: f32, nz: f32, mi: i32, id: i32) -> Self {
        Self {
            position: [vx, vy, vz, 1.0],
            normal: [nx, ny, nz, 1.0],
            material_index: mi,
            id: id,
        }
    }
    const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![0=>Float32x4, 1=>Float32x4, 2=>Sint32, 3=>Sint32];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

}


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SnapVertex {
    pub position: [f32; 4],
}

impl SnapVertex {
    pub fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 1.0],
        }
    }
    pub fn new(vx: f32, vy: f32, vz: f32) -> Self {
        Self {
            position: [vx, vy, vz, 1.0],
        }
    }
    const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0=>Float32x4];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SnapVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}


#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshDrawIndexedIndirect {
    pub vertex_count: u32,
    // The number of vertices to draw.
    pub instance_count: u32,
    // The number of instances to draw.
    pub base_index: u32,
    // The base index within the index buffer.
    pub vertex_offset: u32,
    // The value added to the vertex index before indexing into the vertex buffer.
    pub base_instance: u32, // The instance ID of the first instance to draw.
}