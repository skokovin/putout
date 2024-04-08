use std::ops::Sub;
use cgmath::{InnerSpace, Point3, Vector3};
use serde::{Deserialize, Serialize};

use truck_base::bounding_box::BoundingBox;
use crate::scene::RawMesh;
use crate::shared::{ANGLE_SUBDIVISIONS, SPHERE_PRIME_TYPE, Triangle, TriMesh};



#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpherePrimitive {
    pub id: i32,
    pub center: Point3<f32>,
    pub radius: f32,
    pub mat_idx: i32,
}

impl SpherePrimitive {
    pub fn new(id: i32, center: Point3<f32>, radius: f32,mat_idx: i32) -> Self {
        Self {
            id: id,
            center: center,
            radius: radius,
            mat_idx:mat_idx,
        }
    }
    pub fn triangulate(&self) -> RawMesh {
        let mut triangles: Vec<Triangle> = vec![];
        let sphere = sphere(self.radius, self.center.x, self.center.y, self.center.z);
        let vrtxs = sphere.positions;
        let normals = sphere.normals;
        let mut indx: Vec<i32> = vec![];
        let mut vs: Vec<f32> = vec![];
        let mut counter = 0;
        let bbx: BoundingBox<Point3<f64>> = {
            let pmin = Point3::new((self.center.x - self.radius) as f64, (self.center.y - self.radius) as f64, (self.center.z - self.radius) as f64);
            let pmax = Point3::new((self.center.x + self.radius) as f64, (self.center.y + self.radius) as f64, (self.center.z + self.radius) as f64);
            BoundingBox::from_iter([pmin, pmax])
        };
        sphere.indices.iter().for_each(|i| {
            let vrtx = vrtxs[*i as usize];
            let normal = normals[*i as usize];
            vs.push(vrtx.x);
            vs.push(vrtx.y);
            vs.push(vrtx.z);
            vs.push(normal.x);
            vs.push(normal.y);
            vs.push(normal.z);
            indx.push(counter);
            counter = counter + 1;
        });
        sphere.indices.chunks(3).for_each(|tri|{
            let p0=vrtxs[tri[0] as usize];
            let p1=vrtxs[tri[1] as usize];
            let p2=vrtxs[tri[2] as usize];
            let triangle: Triangle =Triangle::new(
                Point3::new(p0.x,p0.y,p0.z),
                Point3::new(p1.x,p1.y,p1.z),
                Point3::new(p2.x,p2.y,p2.z),
            );
            triangles.push(triangle);
        });

        let rm = RawMesh {
            id: self.id,
            ty:SPHERE_PRIME_TYPE,
            name: self.id.to_string(),
            vertex_normal: vs,
            indx: indx,
            color_indx: self.mat_idx,
            bbx: bbx,
            bvh_index: self.id as usize,
            triangles:triangles,

        };
        rm
    }
}

fn sphere(r: f32, cx: f32, cy: f32, cz: f32) -> TriMesh {
    let mut positions: Vec<Vector3<f32>> = Vec::new();
    let mut indices: Vec<i32> = Vec::new();
    let mut normals: Vec<Vector3<f32>> = Vec::new();
    let center: Vector3<f32> =Vector3::new(cx, cy, cz);

    positions.push(Vector3::new(cx, cy, r + cz));
    normals.push(Vector3::new(cx, cy, r + cz).sub(center).normalize());

    for j in 0..ANGLE_SUBDIVISIONS * 2 {
        let j1 = (j + 1) % (ANGLE_SUBDIVISIONS * 2);
        indices.push(0);
        indices.push((1 + j) as i32);
        indices.push((1 + j1) as i32);
    }

    for i in 0..ANGLE_SUBDIVISIONS - 1 {
        let theta = std::f32::consts::PI * (i + 1) as f32 / ANGLE_SUBDIVISIONS as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        let i0 = 1 + i * ANGLE_SUBDIVISIONS * 2;
        let i1 = 1 + (i + 1) * ANGLE_SUBDIVISIONS * 2;

        for j in 0..ANGLE_SUBDIVISIONS * 2 {
            let phi = std::f32::consts::PI * j as f32 / ANGLE_SUBDIVISIONS as f32;
            let x = sin_theta * phi.cos() * r + cx;
            let y = sin_theta * phi.sin() * r + cy;
            let z = cos_theta * r + cz;
            positions.push(Vector3::new(x, y, z));
            normals.push(Vector3::new(x, y, z).sub(center).normalize());

            if i != ANGLE_SUBDIVISIONS - 2 {
                let j1 = (j + 1) % (ANGLE_SUBDIVISIONS * 2);
                indices.push((i0 + j) as i32);
                indices.push((i1 + j1) as i32);
                indices.push((i0 + j1) as i32);
                indices.push((i1 + j1) as i32);
                indices.push((i0 + j) as i32);
                indices.push((i1 + j) as i32);
            }
        }
    }

    positions.push(Vector3::new(cx, cy, -r + cz));
    normals.push(Vector3::new(cx, cy, -r + cz).sub(center).normalize());

    let i = 1 + (ANGLE_SUBDIVISIONS - 2) * ANGLE_SUBDIVISIONS * 2;
    for j in 0..ANGLE_SUBDIVISIONS * 2 {
        let j1 = (j + 1) % (ANGLE_SUBDIVISIONS * 2);


            indices.push((i + j) as i32);
            indices.push(((ANGLE_SUBDIVISIONS - 1) * ANGLE_SUBDIVISIONS * 2 + 1) as i32);
            indices.push((i + j1) as i32);



    }

    TriMesh {
        indices: indices,
        positions: positions,
        normals: normals,
    }
}






