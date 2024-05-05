use std::{iter, mem};


use std::mem::size_of;
use std::ops::{Range};
use std::rc::Rc;
use cgmath::{Point3};

use itertools::Itertools;
use log::{info, warn};


use parking_lot::{RwLock};







use wgpu::{Adapter, BindGroup, CommandEncoder, COPY_BYTES_PER_ROW_ALIGNMENT, Device, Extent3d, Features, Instance, LoadOp, Operations, Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Texture, TextureFormat, TextureView, TextureViewDescriptor};
use winit::dpi::PhysicalSize;

use crate::device::message_controller::{MessageController, SnapMode};
use crate::device::window_state::WindowState;
use crate::shared::dimension::Dimension;

use crate::shared::screen_capture::ScreenCapture;

pub const BACKGROUND_COLOR1: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.3,
    a: 1.0,
};
pub const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
};

pub const BACKGROUND_COLOR_OLD: wgpu::Color = wgpu::Color {
    r: 0.95703125,
    g: 0.95703125,
    b: 0.859375,
    a: 1.0,
};

pub const HULL_ENABLE: bool = true;
pub const CABLE_ENABLE: bool = true;
pub const CABLE_HILIGHT_ENABLE: bool = true;

pub struct DeviceState {
    pub instance: Rc<RwLock<Instance>>,
    pub adapter: Rc<RwLock<Adapter>>,
    pub device: Rc<RwLock<Device>>,
    pub queue: Rc<RwLock<Queue>>,
    pub is_multi_draw_indirect: bool,
    pub mc: Rc<RwLock<MessageController>>,
    pub screen_capture: ScreenCapture,
}



impl DeviceState {
    pub fn new(mc: Rc<RwLock<MessageController>>, instance: Rc<RwLock<Instance>>, adapter: Rc<RwLock<Adapter>>,
               device: Rc<RwLock<Device>>, queue: Rc<RwLock<Queue>>) -> Self {
        let is_multi_draw_indirect = adapter.clone().read().features().contains(Features::MULTI_DRAW_INDIRECT);
        let screen_capture = ScreenCapture::new(device.clone());

        Self {
            instance: instance,
            adapter: adapter,
            device: device,
            queue: queue,
            is_multi_draw_indirect: is_multi_draw_indirect,
            mc: mc,
            screen_capture: screen_capture,
        }
    }

    #[inline]
    pub fn request_redraw(&self) {}

    #[inline]
    pub fn render(&mut self, _ws:Rc<RwLock<WindowState>>) {

        let ws = _ws.read();
        let scale_factor: f64 = ws.get_scale_factor();
        self.update_shared_buffers(scale_factor);
        let device = self.device.read();
        let queue = self.queue.read();
        match ws.surface.get_current_texture() {
            Ok(out) => {
                let gw = out.texture.width();
                let gh = out.texture.height();
                let mc = self.mc.read();
                let pl = ws.mesh_pipeline.read();
                let bg: BindGroup = pl.bind_mesh_group(&device, &mc.shared_buffers);

                let texture_view_descriptor = TextureViewDescriptor::default();
                let view: TextureView = out.texture.create_view(&texture_view_descriptor);
                let depth_texture: Texture = device.create_texture(&wgpu::TextureDescriptor {
                    size: wgpu::Extent3d {
                        width: gw,
                        height: gh,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth32Float,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    label: None,
                    view_formats: &vec![],
                });
                let depth_view: TextureView = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
                let handl=ws.smaa_target.clone();
                let mut handl_w =handl.write();
                let smaa_frame=handl_w.start_frame(&device,&queue,&view);

                let mut encoder: CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder D"),
                });

                {
                    let _render_pass: RenderPass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass1"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &smaa_frame,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(BACKGROUND_COLOR),
                                store: StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                }

                //SCENE_RENDERING
                mc.scene_state.gpu_mems.iter().for_each(|mem|{
                    if(mem.is_renderable){
                        let mut render_pass: RenderPass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Render Pass HULL"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &smaa_frame,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                view: &depth_view,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: StoreOp::Store,
                                }),
                                stencil_ops: None,
                            }),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });
                        render_pass.set_pipeline(&pl.mesh_render_pipeline);
                        render_pass.set_bind_group(0, &bg, &[]);
                        render_pass.set_vertex_buffer(0, mem.v_buffer.slice(..));
                        render_pass.set_index_buffer(mem.i_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        let indx_count = (mem.i_buffer.size() / mem::size_of::<i32>() as u64) as u32;
                        render_pass.draw_indexed(Range { start: 0, end: indx_count }, 0, Range { start: 0, end: 1 });
                    }
                });
                //snap rendering
                {
                    let bs: BindGroup = pl.bind_snap_group(&device, &mc.shared_buffers);
                    let mut render_pass: RenderPass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass SNAP"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &smaa_frame,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    render_pass.set_pipeline(&pl.snap_render_pipeline);
                    render_pass.set_bind_group(0, &bs, &[]);
                    queue.write_buffer(&mc.scene_state.snap_vertex_buffer, 0, bytemuck::cast_slice(&[0.0, 0.0, 0.0, 1.0]));
                    render_pass.set_vertex_buffer(0, mc.scene_state.snap_vertex_buffer.slice(..));
                    render_pass.draw(Range { start: 0, end: 6 }, Range { start: 0, end: 1 });
                }

                //text_rendering
                let text_layput_handler=mc.text_layout.clone();
                let text_layput =text_layput_handler.write();
                {
                    let mut pass: RenderPass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &smaa_frame,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    text_layput.text_renderer.render(&text_layput.atlas, &mut pass).unwrap();
                }

                queue.submit(iter::once(encoder.finish()));
                smaa_frame.resolve();
                out.present();
            }
            Err(e) => { println!("{}", e) }
        }
        self.screen_capture.refresh(self.mc.clone());
    }

    #[inline]
    pub fn capture_screen(&mut self, _ws: &RwLock<WindowState>) {
        if !self.screen_capture.is_captured() {
            let ws = _ws.read();
            let mc = self.mc.read();
            let window_size: PhysicalSize<f32> = ws.get_window_size();
            let w = window_size.width as u32;
            let h = window_size.height as u32;
            let _sf = ws.get_scale_factor() as f32;

            let sel_texture_desc = wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TextureFormat::Rgba32Sint,
                usage: wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: None,
                view_formats: &[],
            };
            let sel_texture: Texture = self.device.read().create_texture(&sel_texture_desc);

            let sel_texture_view: TextureView = sel_texture.create_view(&Default::default());
            let sel_depth_texture: Texture = self.device.read().create_texture(&wgpu::TextureDescriptor {
                size: Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: None,
                view_formats: &[],
            });
            let sel_depth_view: TextureView = sel_depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut sel_encoder: CommandEncoder = self.device.read().create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: Some("Selection Encoder D") });
            let pl = ws.mesh_pipeline.read();
            let bg: BindGroup = pl.bind_mesh_group(&self.device.read(), &mc.shared_buffers);
            let queue = self.queue.read();
            let mut is_done=false;

            {
                let _render_pass: RenderPass = sel_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass1"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &sel_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(BACKGROUND_COLOR),
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &sel_depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            }

            //DRAW TO BUFFER
            mc.scene_state.gpu_mems.iter().for_each(|mem|{
                if(mem.is_renderable){
                    {   let indx_count = (mem.i_buffer.size() / mem::size_of::<i32>() as u64) as u32;
                        let mut sel_render_pass: RenderPass = sel_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Render Pass HULL"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &sel_texture_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                view: &sel_depth_view,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: StoreOp::Store,
                                }),
                                stencil_ops: None,
                            }),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                        sel_render_pass.set_pipeline(&pl.selection_render_pipeline);
                        sel_render_pass.set_bind_group(0, &bg, &[]);
                        sel_render_pass.set_vertex_buffer(0, mem.v_buffer.slice(..));
                        sel_render_pass.set_index_buffer(mem.i_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        sel_render_pass.draw_indexed(Range { start: 0, end: indx_count }, 0, Range { start: 0, end: 1 });
                        is_done=true;
                    }
                }
            });
            //Copy Buffer to Host
            if(is_done){
                {
                    let image_width = COPY_BYTES_PER_ROW_ALIGNMENT * (w / COPY_BYTES_PER_ROW_ALIGNMENT);

                    let sc = self.screen_capture.get_capture_buffer(self.device.clone(), w as usize, h as usize, image_width as usize);
                    sel_encoder.copy_texture_to_buffer(
                        wgpu::ImageCopyTexture {
                            aspect: wgpu::TextureAspect::All,
                            texture: &sel_texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: 0,
                                z: 0,
                            },
                        },
                        wgpu::ImageCopyBuffer {
                            buffer: &sc.write(),
                            layout: wgpu::ImageDataLayout {
                                offset: 0,
                                bytes_per_row: Some(image_width * 4 * 4),
                                rows_per_image: Some(h),
                            },
                        },
                        Extent3d {
                            width: image_width,
                            height: h,
                            depth_or_array_layers: 1,
                        },
                    );
                    queue.submit(iter::once(sel_encoder.finish()));
                    println!("IMAGE DONE WIDDT={}", image_width);
                    self.screen_capture.copy_to_host();
                }
            }

        } else {
            self.screen_capture.refresh(self.mc.clone());
        }
    }

    fn update_shared_buffers(&self, scale_factor: f64) {
        let slicer_is_dirty = self.mc.read().scene_state.slicer.is_dirty;
        let materials_is_dirty = self.mc.read().is_materials_dirty;
        let _snap_mode: SnapMode = self.mc.read().snap_mode.clone();

        let is_metadata0_dirty = self.mc.read().scene_state.gpu_mems[0].is_metadata_dirty;
        let is_metadata1_dirty = self.mc.read().scene_state.gpu_mems[1].is_metadata_dirty;
        let is_metadata2_dirty = self.mc.read().scene_state.gpu_mems[2].is_metadata_dirty;
        let is_metadata3_dirty = self.mc.read().scene_state.gpu_mems[3].is_metadata_dirty;
        let is_metadata4_dirty = self.mc.read().scene_state.gpu_mems[4].is_metadata_dirty;
        let is_metadata5_dirty = self.mc.read().scene_state.gpu_mems[5].is_metadata_dirty;
        let is_metadata6_dirty = self.mc.read().scene_state.gpu_mems[6].is_metadata_dirty;
        let is_metadata7_dirty = self.mc.read().scene_state.gpu_mems[7].is_metadata_dirty;

        if is_metadata0_dirty {
            let hmd = &self.mc.read().scene_state.gpu_mems[0].metadata.clone();
            self.mc.write().shared_buffers.update_metadata0(self.device.clone(), self.queue.clone(), hmd);
            self.mc.write().scene_state.gpu_mems[0].reset_dirty_metadata();
        }

        if is_metadata1_dirty {
            let hmd = &self.mc.read().scene_state.gpu_mems[1].metadata.clone();
            self.mc.write().shared_buffers.update_metadata1(self.device.clone(), self.queue.clone(), hmd);
            self.mc.write().scene_state.gpu_mems[1].reset_dirty_metadata();
        }

        if is_metadata2_dirty {
            let hmd = &self.mc.read().scene_state.gpu_mems[2].metadata.clone();
            self.mc.write().shared_buffers.update_metadata2(self.device.clone(), self.queue.clone(), hmd);
            self.mc.write().scene_state.gpu_mems[2].reset_dirty_metadata();
        }

        if is_metadata3_dirty {
            let hmd = &self.mc.read().scene_state.gpu_mems[3].metadata.clone();
            self.mc.write().shared_buffers.update_metadata3(self.device.clone(), self.queue.clone(), hmd);
            self.mc.write().scene_state.gpu_mems[3].reset_dirty_metadata();
        }

        if is_metadata4_dirty {
            let hmd = &self.mc.read().scene_state.gpu_mems[4].metadata.clone();
            self.mc.write().shared_buffers.update_metadata4(self.device.clone(), self.queue.clone(), hmd);
            self.mc.write().scene_state.gpu_mems[4].reset_dirty_metadata();
        }

        if is_metadata5_dirty {
            let hmd = &self.mc.read().scene_state.gpu_mems[5].metadata.clone();
            self.mc.write().shared_buffers.update_metadata5(self.device.clone(), self.queue.clone(), hmd);
            self.mc.write().scene_state.gpu_mems[5].reset_dirty_metadata();
        }
        if is_metadata6_dirty {
            let hmd = &self.mc.read().scene_state.gpu_mems[6].metadata.clone();
            self.mc.write().shared_buffers.update_metadata6(self.device.clone(), self.queue.clone(), hmd);
            self.mc.write().scene_state.gpu_mems[6].reset_dirty_metadata();
        }
        if is_metadata7_dirty {
            let hmd = &self.mc.read().scene_state.gpu_mems[7].metadata.clone();
            self.mc.write().shared_buffers.update_metadata7(self.device.clone(), self.queue.clone(), hmd);
            self.mc.write().scene_state.gpu_mems[7].reset_dirty_metadata();
        }


        if materials_is_dirty {
            self.mc.write().reset_material_dirty();
            let mats = &self.mc.read().materials;
            let mcw = self.mc.read();
            mcw.shared_buffers.update_material(self.queue.clone(), mats);
        }


        if slicer_is_dirty {
            let sp = self.mc.read().scene_state.slicer.slice_positions();
            let arr: [f32; 6] = [sp.0, sp.1, sp.2, sp.3, sp.4, sp.5];
            self.mc.write().shared_buffers.update_slicer(self.queue.clone(), &arr);
            self.mc.write().scene_state.slicer.reset_dirty();
        };

        {
            let active_point: Point3<f32> = self.mc.read().active_point.clone();
            let dimension: Dimension = self.mc.read().dimension.clone();
            let snap_mode: SnapMode = self.mc.read().snap_mode.clone();
            self.mc.write().shared_buffers.update_snap(self.queue.clone(), active_point, dimension,snap_mode);
        }


        {
            let mvp = self.mc.read().scene_state.camera.get_mvp_buffer().clone();
            let norms = self.mc.read().scene_state.camera.get_norm_buffer().clone();
            let forw = self.mc.read().scene_state.camera.get_forward_dir_buffer();
            self.mc.write().shared_buffers.update_camera(self.queue.clone(), &mvp, &norms, &forw);
        }

        {
            let eye = *self.mc.read().scene_state.camera.eye.clone().read();
            let light_position: &[f32; 3] = eye.as_ref();
            let eye_position: &[f32; 3] = eye.as_ref();
            #[cfg(not(target_arch = "wasm32"))]
                let w = self.mc.read().scene_state.camera.screen_w;
            #[cfg(not(target_arch = "wasm32"))]
                let h = self.mc.read().scene_state.camera.screen_h;
            #[cfg(target_arch = "wasm32")]
                let w = self.mc.read().scene_state.camera.screen_w / scale_factor as f32;
            #[cfg(target_arch = "wasm32")]
                let h = self.mc.read().scene_state.camera.screen_h / scale_factor as f32;

            self.mc.write().shared_buffers.update_lights(self.queue.clone(), light_position, eye_position, w, h);
        }
    }
}

struct BufferDimensions {
    width: usize,
    height: usize,
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
}

impl BufferDimensions {
    fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
}

