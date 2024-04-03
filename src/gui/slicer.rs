use cgmath::Point3;
use truck_base::bounding_box::BoundingBox;

pub struct Slicer {
    pub x_max_sensor: f32,
    pub x_min_sensor: f32,
    pub y_max_sensor: f32,
    pub y_min_sensor: f32,
    pub z_max_sensor: f32,
    pub z_min_sensor: f32,

    pub x_max_plane: f32,
    pub x_min_plane: f32,
    pub y_max_plane: f32,
    pub y_min_plane: f32,
    pub z_max_plane: f32,
    pub z_min_plane: f32,

    pub is_dirty: bool,

}

impl Slicer {
    pub fn new() -> Self {
        Self {
            x_max_sensor: 1.0,
            x_min_sensor: 0.0,
            y_max_sensor: 1.0,
            y_min_sensor: 0.0,
            z_max_sensor: 1.0,
            z_min_sensor: 0.0,
            x_max_plane: 10000.0,
            x_min_plane: -10000.0,
            y_max_plane: 10000.0,
            y_min_plane: -10000.0,
            z_max_plane: 10000.0,
            z_min_plane: -10000.0,
            is_dirty: true,
        }
    }
    pub fn default() -> Self {
        Self {
            x_max_sensor: 1.0,
            x_min_sensor: 0.0,
            y_max_sensor: 1.0,
            y_min_sensor: 0.0,
            z_max_sensor: 1.0,
            z_min_sensor: 0.0,
            x_max_plane: 10000.0,
            x_min_plane: -10000.0,
            y_max_plane: 10000.0,
            y_min_plane: -10000.0,
            z_max_plane: 10000.0,
            z_min_plane: -10000.0,
            is_dirty: true,
        }
    }
    pub fn set_slicer(&mut self, x_max_sensor: f32, x_min_sensor: f32, y_max_sensor: f32, y_min_sensor: f32, z_max_sensor: f32, z_min_sensor: f32) {
        self.x_max_sensor = x_max_sensor;
        self.x_min_sensor = x_min_sensor;
        self.y_max_sensor = y_max_sensor;
        self.y_min_sensor = y_min_sensor;
        self.z_max_sensor = z_max_sensor;
        self.z_min_sensor = z_min_sensor;
        self.is_dirty = true;
    }
    pub fn slice_positions(&self) -> (f32, f32, f32, f32, f32, f32) {
        let dx = (self.x_max_plane - self.x_min_plane).abs();
        let dy = (self.x_max_plane - self.x_min_plane).abs();
        let dz = (self.x_max_plane - self.x_min_plane).abs();

        let pos_x_max = self.x_max_plane - (dx * (1.0 - self.x_max_sensor));
        let pos_x_min = self.x_min_plane + (dx * self.x_min_sensor);
        let pos_y_max = self.y_max_plane - (dy * (1.0 - self.y_max_sensor));
        let pos_y_min = self.y_min_plane + (dy * self.y_min_sensor);
        let pos_z_max = self.z_max_plane - (dz * (1.0 - self.z_max_sensor));
        let pos_z_min = self.z_min_plane + (dz * self.z_min_sensor);

        (pos_x_max, pos_x_min, pos_y_max, pos_y_min, pos_z_max, pos_z_min)
    }
    pub fn set_by_bbx(&mut self, bbx: &BoundingBox<Point3<f64>>) {
        self.x_max_sensor = 1.0;
        self.x_min_sensor = 0.0;
        self.y_max_sensor = 1.0;
        self.y_min_sensor = 0.0;
        self.z_max_sensor = 1.0;
        self.z_min_sensor = 0.0;
        let min = bbx.min();
        let max = bbx.max();
        self.x_max_plane = max.x as f32;
        self.x_min_plane = min.x as f32;
        self.y_max_plane = max.y as f32;
        self.y_min_plane = min.y as f32;
        self.z_max_plane = max.z as f32;
        self.z_min_plane = min.z as f32;
        self.is_dirty = true
    }
    pub fn reset_dirty(&mut self) {
        self.is_dirty = false;
    }
}