use cgmath::num_traits::Float;
use cgmath::{MetricSpace, Point3};
use crate::remote::hull_state;

#[derive(PartialEq, Clone)]
pub enum DimensionMode {
    Point = 0,
    Line = 1,
    Angle = 2,
    NotSet = 3,
}

#[derive(Clone)]
pub struct Dimension {
    pub p0: Point3<f32>,
    pub p1: Point3<f32>,
    pub p2: Point3<f32>,
    pub p3: Point3<f32>,
    pub mode: DimensionMode,
}

impl Dimension {
    pub fn new() -> Self {
        Self {
            p0: Point3::new(f32::max_value(), f32::max_value(), f32::max_value()),
            p1: Point3::new(f32::max_value(), f32::max_value(), f32::max_value()),
            p2: Point3::new(f32::max_value(), f32::max_value(), f32::max_value()),
            p3: Point3::new(f32::max_value(), f32::max_value(), f32::max_value()),
            mode: DimensionMode::NotSet,
        }
    }

    pub fn set_point(&mut self, p:Point3<f32>, mode:DimensionMode){
       match  mode{
           DimensionMode::Point => {
               self.p0=p.clone();
               self.mode=DimensionMode::Point
           }
           DimensionMode::Line => {
               if self.mode==DimensionMode::Line {
                   self.p0.x=f32::max_value();
                   println!("RESET LINE");
               }
               if self.p0.x==f32::max_value() {
                   self.p0=p.clone();
                   self.mode=DimensionMode::NotSet;
                   #[cfg(target_arch = "wasm32")]
                   hull_state::dim_set_fist_point(web_sys::js_sys::Float32Array::from((vec![self.p0.x*10.0,self.p0.y*10.0,self.p0.z*10.0]).as_slice()));
                   println!("P0 LINE is {:?}",self.p0);
               }else{
                   self.p1=p.clone();
                   self.mode=DimensionMode::Line;
                   let dist=self.p1.distance(self.p0);
                   #[cfg(target_arch = "wasm32")]
                   hull_state::dim_set_second_point(web_sys::js_sys::Float32Array::from(vec![self.p1.x*10.0,self.p1.y*10.0,self.p1.z*10.0,dist*10.0].as_slice()));
                   println!("P1 LINE is {:?}",self.p1);
               }
           }
           DimensionMode::Angle => {}
           DimensionMode::NotSet => {}
       }
    }

    pub fn clear(&mut self){
        self.p0= Point3::new(f32::max_value(), f32::max_value(), f32::max_value());
        self.p1= Point3::new(f32::max_value(), f32::max_value(), f32::max_value());
        self.p2= Point3::new(f32::max_value(), f32::max_value(), f32::max_value());
        self.p3= Point3::new(f32::max_value(), f32::max_value(), f32::max_value());
        self.mode= DimensionMode::NotSet;
    }


}