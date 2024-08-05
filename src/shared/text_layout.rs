use std::rc::Rc;
use std::sync::Arc;
use cgmath::num_traits::Float;
use cgmath::{EuclideanSpace, Matrix4, MetricSpace, Point3, Vector2, Vector4};
use glyphon::{Attrs, Buffer, Cache, Color, Family, fontdb, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport};
use glyphon::fontdb::{Source};

use parking_lot::RwLock;

use wgpu::{Device, MultisampleState, Queue, SurfaceConfiguration};
use winit::dpi::PhysicalPosition;
use crate::device::message_controller::SnapMode;
use crate::shared::dimension::{Dimension, DimensionMode};

const SANSSERIF: &[u8] = include_bytes!("../fonts/ARIALUNI.TTF");

struct FontSource {
    font_bytes: Vec<u8>,
}

impl FontSource {
    pub fn new(bytes: &[u8]) -> Self {
        let mut v: Vec<u8> = vec![];
        v.extend_from_slice(bytes);
        Self {
            font_bytes: v
        }
    }

    pub fn get_comic() -> Self {
        let mut v: Vec<u8> = vec![];
        v.extend_from_slice(SANSSERIF);
        Self {
            font_bytes: v
        }
    }

    pub fn generate_fonts() -> Vec<fontdb::Source> {
        let mut fonts: Vec<fontdb::Source> = vec![];
        let sans = Source::Binary(Arc::new(FontSource::new(SANSSERIF)));
        fonts.push(sans);
        fonts
    }
}

impl AsRef<[u8]> for FontSource {
    fn as_ref(&self) -> &[u8] {
        self.font_bytes.as_slice()
    }
}

unsafe impl Send for FontSource {}

unsafe impl Sync for FontSource {}


pub struct TextLayout {
    is_dirty: bool,
    width: u32,
    height: u32,
    device: Rc<RwLock<Device>>,
    queue: Rc<RwLock<Queue>>,
    pub text_renderer: TextRenderer,
    pub atlas: TextAtlas,
    cache: SwashCache,
    pub viewport: Viewport,
    font_system: FontSystem,
    snap_symbol_buff: Buffer,
    snap_value_buff: Buffer,
    dim_buff: Buffer,
    snap_mode: SnapMode,
    pub active_id: i32,
    pub active_point: Point3<f32>,
    scale_factor: f64,
    dimension_value: f32,
    dimension_pos: Vector2<f32>,
}

impl TextLayout {
    pub fn new(device: Rc<RwLock<Device>>, queue: Rc<RwLock<Queue>>, surface_config: SurfaceConfiguration) -> Self {
        let width = surface_config.width;
        let height = surface_config.height;
        let mut swash_cache: SwashCache = SwashCache::new();
        let cache = Cache::new(&device.read());
        let mut viewport: Viewport = Viewport::new(&device.read(), &cache);
        let mut atlas: TextAtlas = TextAtlas::new(&device.read() , &queue.read(), &cache, surface_config.format);
        let mut text_renderer: TextRenderer = TextRenderer::new(&mut atlas, &device.read(), MultisampleState::default(), None);
        let mut font_system: FontSystem = FontSystem::new_with_fonts(FontSource::generate_fonts());

        let mut snap_symbol_buff: Buffer = glyphon::Buffer::new(&mut font_system, Metrics::new(15.0, 16.0));
        snap_symbol_buff.set_size(&mut font_system, Some(20.0), Some(200.0));
        snap_symbol_buff.set_text(&mut font_system, "", Attrs::new().family(Family::SansSerif), Shaping::Advanced);
        snap_symbol_buff.shape_until_scroll(&mut font_system, false);


        let mut snap_value_buff: Buffer = glyphon::Buffer::new(&mut font_system, Metrics::new(15.0, 15.0));
        snap_value_buff.set_size(&mut font_system, Some(200.0), Some(200.0));
        snap_value_buff.set_text(&mut font_system, "BBBBBB", Attrs::new().family(Family::Name("Arial")), Shaping::Basic);
        snap_value_buff.shape_until_scroll(&mut font_system, false);


        let mut dim_buff: Buffer = glyphon::Buffer::new(&mut font_system, Metrics::new(45.0, 45.0));
        dim_buff.set_size(&mut font_system, Some(200.0), Some(200.0));
        dim_buff.set_text(&mut font_system, "", Attrs::new().family(Family::Name("Arial")), Shaping::Basic);
        dim_buff.shape_until_scroll(&mut font_system, false);


        text_renderer.prepare(
            &device.read(),
            &queue.read(),
            &mut font_system,
            &mut atlas,
            &viewport,
            [],
            &mut swash_cache,
        ).unwrap();

        Self {
            is_dirty: false,
            width: width,
            height: height,
            device: device.clone(),
            queue: queue.clone(),
            text_renderer: text_renderer,
            atlas: atlas,
            cache: swash_cache,
            viewport:viewport,
            font_system: font_system,
            snap_symbol_buff: snap_symbol_buff,
            snap_value_buff: snap_value_buff,
            dim_buff: dim_buff,
            snap_mode: SnapMode::NotSet,
            active_id: 0,
            active_point: Point3::new(0.0, 0.0, 0.0),
            scale_factor: 1.0,
            dimension_value: 0.0,
            dimension_pos: Vector2::new(0.0, 0.0),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32, scale_factor: f64) {


        self.width = width;
        self.height = height;
        self.set_scale_factor(scale_factor);
        let resol=Resolution {
            width: (self.width as f64 / self.scale_factor) as u32,
            height: (self.height as f64 / self.scale_factor) as u32,
        };
        self.viewport.update(
            &self.queue.read(),
            resol
        );
        self.is_dirty = true;
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = 1.0
    }
    #[cfg(target_arch = "wasm32")]
    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor
    }

    pub fn on_render(&mut self,
                     snap_mode: SnapMode,
                     active_id: i32,
                     active_point: Point3<f32>,
                     mouse_position: PhysicalPosition<f64>,
                     dimension: Dimension,
                     mvp: Matrix4<f32>) {
        if snap_mode != self.snap_mode {
            self.snap_mode = snap_mode;
            self.gen_snap_mode_area();
            self.is_dirty = true;
        }


        if self.dimension_value > 0.0 {
            self.gen_dim_area();
            self.is_dirty = true;
        }

        if self.active_id != active_id || self.active_point != active_point {
            self.active_id = active_id;
            self.active_point = active_point;
            self.gen_snap_value_area();
            self.is_dirty = true;
        }


        if self.is_dirty {
            self.refresh(mouse_position);
            self.is_dirty = false;
        }

        match dimension.mode {
            DimensionMode::Point => {
                self.dimension_value = 0.0;
            }
            DimensionMode::Line => {
                let midpoint: Vector4<f32> = dimension.p0.midpoint(dimension.p1).to_homogeneous();
                let p2_d = mvp * midpoint;
                self.dimension_pos.x = ((p2_d.x / p2_d.w + 1.0) * self.width as f32) / 2.0;
                self.dimension_pos.y = ((p2_d.y / p2_d.w - 1.0) * self.height as f32) / -2.0;
                let dist = dimension.p0.distance(dimension.p1);
                if dist < 0.3
                    || self.dimension_pos.x < 0.0
                    || self.dimension_pos.y < 0.0
                    || self.dimension_pos.x > self.width as f32
                    || self.dimension_pos.y > self.height as f32 {
                    self.dimension_value = 0.0;
                } else {
                    self.dimension_value = dist * 10.0;
                }
                // println!("{} {} {}", self.dimension_pos.x, self.dimension_pos.y, self.dimension_value);
            }
            DimensionMode::Angle => {
                self.dimension_value = 0.0;
            }
            DimensionMode::NotSet => {
                self.dimension_value = 0.0;
            }
        }
    }

    fn refresh(&mut self, mouse_position: PhysicalPosition<f64>) {
        let snap_symbol_area = TextArea {
            buffer: &self.snap_symbol_buff,
            left: (self.width as f64 / self.scale_factor - 100.0) as f32,
            top: (10.0 / self.scale_factor) as f32,
            scale: 1.0,
            bounds: TextBounds {
                left: (self.width as f64 / self.scale_factor - 100.0) as i32,
                top: 0,
                right: (self.width as f64 / self.scale_factor) as i32,
                bottom: (self.height as f64 / self.scale_factor + 300.0) as i32,
            },
            default_color: Color::rgb(255, 255, 255),
        };

        let snap_value_area = TextArea {
            buffer: &self.snap_value_buff,
            left: (mouse_position.x / self.scale_factor + 20.0) as f32,
            top: (mouse_position.y / self.scale_factor - 30.0) as f32,
            scale: 1.0,
            bounds: TextBounds {
                left: (mouse_position.x / self.scale_factor) as i32,
                top: (mouse_position.y / self.scale_factor - 30.0) as i32,
                right: (mouse_position.x / self.scale_factor) as i32 + 500,
                bottom: (mouse_position.y / self.scale_factor) as i32 + 500,
            },
            default_color: Color::rgb(255, 255, 255),
        };

        let mut text_areas: Vec<TextArea> = vec![];
        text_areas.push(snap_symbol_area);
        text_areas.push(snap_value_area);

        if self.dimension_value > 0.0 {
            let dim_area = TextArea {
                buffer: &self.dim_buff,
                left: (self.dimension_pos.x / self.scale_factor as f32) as f32,
                top: (self.dimension_pos.y / self.scale_factor as f32) as f32,
                scale: 1.0,
                bounds: TextBounds {
                    left: (self.dimension_pos.x / self.scale_factor as f32) as i32,
                    top: (self.dimension_pos.y / self.scale_factor as f32) as i32,
                    right: (self.dimension_pos.x / self.scale_factor as f32) as i32 + 500,
                    bottom: (self.dimension_pos.y / self.scale_factor as f32) as i32 + 500,
                },
                default_color: Color::rgb(255, 0, 0),
            };
            text_areas.push(dim_area);
        }

/*        let resol=Resolution {
            width: (self.width as f64 / self.scale_factor) as u32,
            height: (self.height as f64 / self.scale_factor) as u32,
        };
        self.viewport.update(
            &self.queue.read(),
            resol
        );*/

        self.text_renderer.prepare(
            &self.device.read(),
            &self.queue.read(),
            &mut self.font_system,
            &mut self.atlas,
            &mut self.viewport,
            text_areas,
            &mut self.cache,
        ).unwrap();
    }

    fn gen_snap_mode_area(&mut self) {
        let txt = match self.snap_mode {
            SnapMode::Vertex => { String::from("V") }
            SnapMode::Edge => { String::from("E") }
            SnapMode::Face => { String::from("F") }
            SnapMode::Disabled => { String::from("O") }
            SnapMode::LineDim => { String::from("L") }
            SnapMode::NotSet => { String::from("N") }
        };
        self.snap_symbol_buff.set_text(&mut self.font_system, txt.as_str(), Attrs::new().family(Family::SansSerif), Shaping::Advanced);
    }

    fn gen_snap_value_area(&mut self) {
        if self.active_id != 0 {
            let id = self.active_id.to_string();
            let x = format!("{:6.3}", self.active_point.x / 100.0);
            let y = format!("{:6.3}", self.active_point.y / 100.0);
            let z = format!("{:6.3}", self.active_point.z / 100.0);
            let str = if self.active_point.x < f32::max_value() {
                format!("ID {} \n X {}\n Y {} \n Z {}", id, x, y, z)
            } else {
                "".to_string()
            };
            self.snap_value_buff.set_text(&mut self.font_system, str.as_str(), Attrs::new().family(Family::Name("Arial")), Shaping::Basic);
        } else {
            self.snap_value_buff.set_text(&mut self.font_system, "", Attrs::new().family(Family::SansSerif), Shaping::Advanced);
        }
    }

    fn gen_dim_area(&mut self) {
        let txt = self.dimension_value.floor().to_string();
        self.dim_buff.set_text(&mut self.font_system, txt.as_str(), Attrs::new().family(Family::SansSerif), Shaping::Advanced);
    }

    pub fn clear_dimension_value(&mut self){
        self.dimension_value = 0.0;
    }
}