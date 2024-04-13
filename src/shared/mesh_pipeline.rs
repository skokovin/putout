use std::rc::Rc;
use parking_lot::{RwLock};
use wgpu::{BindGroup, BindGroupLayout, BlendFactor, BlendOperation, Device, Face, FrontFace, PipelineLayout, RenderPipeline, TextureFormat};
use crate::shared::mesh_common::{MeshVertex, SnapVertex};
use crate::shared::shared_buffers::SharedBuffers;

pub struct MeshPipeLine {
    mesh_bind_group_layout: BindGroupLayout,
    pub mesh_render_pipeline: RenderPipeline,

    snap_bind_group_layout: BindGroupLayout,
    pub snap_render_pipeline: RenderPipeline,

    selection_bind_group_layout: BindGroupLayout,
    pub selection_render_pipeline: RenderPipeline,

}

impl MeshPipeLine {
    pub fn new(_device: Rc<RwLock<Device>>, format: TextureFormat) -> Self {
        let device = _device.write();

        //MESH PIPELINE
        let mesh_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Mesh Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
        });
        let mesh_bind_group_layout: BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 11,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 12,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 13,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },


            ],
            label: Some("mesh Bind Group Layout"),
        });
        let mesh_pipeline_layout: PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Mesh Render Pipeline Layout"),
            bind_group_layouts: &[&mesh_bind_group_layout],
            push_constant_ranges: &[],
        });
        let mesh_render_pipeline: RenderPipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Mesh Render Pipeline"),
            layout: Some(&mesh_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &mesh_shader,
                entry_point: "vs_main",
                buffers: &[MeshVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &mesh_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::default(),
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: Default::default(),
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        //SNAP PIPELINE
        let snap_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Snap Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/snap.wgsl").into()),
        });
        let snap_bind_group_layout: BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 11,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 12,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 13,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

            ],
            label: Some("snap Bind Group Layout"),
        });
        let snap_pipeline_layout: PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Snap Render Pipeline Layout"),
            bind_group_layouts: &[&snap_bind_group_layout],
            push_constant_ranges: &[],
        });
        let snap_render_pipeline: RenderPipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Snap Render Pipeline"),
            layout: Some(&snap_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &snap_shader,
                entry_point: "vs_main",
                buffers: &[SnapVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &snap_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::default(),
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: Default::default(),
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        //SELECTION PIPELINE
        let selection_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Selection Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/selection.wgsl").into()),
        });
        let selection_bind_group_layout: BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 11,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 12,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 13,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },

            ],
            label: Some("sel Bind Group Layout"),
        });
        let selection_pipeline_layout: PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Selection Pipeline Layout"),
            bind_group_layouts: &[&selection_bind_group_layout],
            push_constant_ranges: &[],
        });
        let selection_render_pipeline: RenderPipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Selection Pipeline"),
            layout: Some(&selection_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &selection_shader,
                entry_point: "vs_main",
                buffers: &[MeshVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &selection_shader,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: TextureFormat::Rgba32Sint,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                ],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::default(),
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: Default::default(),
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });


        Self {
            mesh_bind_group_layout,
            mesh_render_pipeline: mesh_render_pipeline,
            snap_bind_group_layout: snap_bind_group_layout,
            snap_render_pipeline: snap_render_pipeline,
            selection_bind_group_layout: selection_bind_group_layout,
            selection_render_pipeline: selection_render_pipeline,
        }
    }

    pub fn bind_mesh_group(&self, device: &Device, shred_buffers: &SharedBuffers) -> BindGroup {
        let camera_buffer = shred_buffers.camera_buffer.clone();
        let material_buffer = shred_buffers.material_buffer.clone();
        let light_buffer = shred_buffers.light_buffer.clone();
        let mode_buffer = shred_buffers.mode_buffer.clone();
        let slice_buffer = shred_buffers.slice_buffer.clone();
        let snap_buffer = shred_buffers.snap_buffer.clone();
        let metadata_buffer0 = shred_buffers.metadata_buffer0.clone();
        let metadata_buffer1 = shred_buffers.metadata_buffer1.clone();
        let metadata_buffer2 = shred_buffers.metadata_buffer2.clone();
        let metadata_buffer3 = shred_buffers.metadata_buffer3.clone();
        let metadata_buffer4 = shred_buffers.metadata_buffer4.clone();
        let metadata_buffer5 = shred_buffers.metadata_buffer5.clone();
        let metadata_buffer6 = shred_buffers.metadata_buffer6.clone();
        let metadata_buffer7 = shred_buffers.metadata_buffer7.clone();

        let mesh_uniform_bind_group: BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.mesh_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: material_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: mode_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: slice_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: snap_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: metadata_buffer0.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: metadata_buffer1.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: metadata_buffer2.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: metadata_buffer3.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: metadata_buffer4.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: metadata_buffer5.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: metadata_buffer6.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 13,
                    resource: metadata_buffer7.read().as_entire_binding(),
                },
            ],
            label: Some("Mesh Bind Group"),
        });
        mesh_uniform_bind_group
    }

    pub fn bind_selection_group(&self, device: &Device, shred_buffers: &SharedBuffers) -> BindGroup {
        let camera_buffer = shred_buffers.camera_buffer.clone();
        let material_buffer = shred_buffers.material_buffer.clone();
        let light_buffer = shred_buffers.light_buffer.clone();
        let mode_buffer = shred_buffers.mode_buffer.clone();
        let slice_buffer = shred_buffers.slice_buffer.clone();
        let snap_buffer = shred_buffers.snap_buffer.clone();
        let metadata_buffer0 = shred_buffers.metadata_buffer0.clone();
        let metadata_buffer1 = shred_buffers.metadata_buffer1.clone();
        let metadata_buffer2 = shred_buffers.metadata_buffer2.clone();
        let metadata_buffer3 = shred_buffers.metadata_buffer3.clone();

        let metadata_buffer4 = shred_buffers.metadata_buffer4.clone();
        let metadata_buffer5 = shred_buffers.metadata_buffer5.clone();
        let metadata_buffer6 = shred_buffers.metadata_buffer6.clone();
        let metadata_buffer7 = shred_buffers.metadata_buffer7.clone();

        let selection_uniform_bind_group: BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.selection_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: material_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: mode_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: slice_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: snap_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: metadata_buffer0.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: metadata_buffer1.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: metadata_buffer2.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: metadata_buffer3.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: metadata_buffer4.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: metadata_buffer5.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: metadata_buffer6.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 13,
                    resource: metadata_buffer7.read().as_entire_binding(),
                },
            ],
            label: Some("Selection Bind Group"),
        });
        selection_uniform_bind_group
    }


    pub fn bind_snap_group(&self, device: &Device, shred_buffers: &SharedBuffers) -> BindGroup {
        let camera_buffer = shred_buffers.camera_buffer.clone();
        let material_buffer = shred_buffers.material_buffer.clone();
        let light_buffer = shred_buffers.light_buffer.clone();
        let mode_buffer = shred_buffers.mode_buffer.clone();
        let slice_buffer = shred_buffers.slice_buffer.clone();
        let snap_buffer = shred_buffers.snap_buffer.clone();
        let metadata_buffer0 = shred_buffers.metadata_buffer0.clone();
        let metadata_buffer1 = shred_buffers.metadata_buffer1.clone();
        let metadata_buffer2 = shred_buffers.metadata_buffer2.clone();
        let metadata_buffer3 = shred_buffers.metadata_buffer3.clone();
        let metadata_buffer4 = shred_buffers.metadata_buffer4.clone();
        let metadata_buffer5 = shred_buffers.metadata_buffer5.clone();
        let metadata_buffer6 = shred_buffers.metadata_buffer6.clone();
        let metadata_buffer7 = shred_buffers.metadata_buffer7.clone();
        let snap_uniform_bind_group: BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.snap_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: material_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: mode_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: slice_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: snap_buffer.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: metadata_buffer0.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: metadata_buffer1.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: metadata_buffer2.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: metadata_buffer3.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: metadata_buffer4.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: metadata_buffer5.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: metadata_buffer6.read().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 13,
                    resource: metadata_buffer7.read().as_entire_binding(),
                },
            ],
            label: Some("Snap Bind Group"),
        });
        snap_uniform_bind_group
    }
}
