use crate::unreal::types::structs::{FMinimalViewInfo, FVector};

pub type Matrix4f64 = [f64; 16];

#[derive(Debug, Clone, Copy)]
pub struct Vector4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Default for Vector2 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Vector2 {
    pub fn distance(&self, other: &Vector2) -> f32 {
        ((other.x - self.x).powi(2) + (other.y - self.y).powi(2)).sqrt()
    }

    pub fn to_u32(&self) -> [u32; 2] {
        [self.x as u32, self.y as u32]
    }

    pub fn to_egui(&self) -> egui::Pos2 {
        egui::Pos2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl PartialEq for Vector2 {
    fn eq(&self, other: &Self) -> bool {
        // self == other
        self.x == other.x && self.y == other.y
    }
}

#[flamer::flame]
pub fn world2screen(position: FVector, pov: FMinimalViewInfo) -> Vector2 {
    let view_matrix = pov.rotation.to_matrix();

    let v_axis_x = FVector {
        x: view_matrix[0][0],
        y: view_matrix[0][1],
        z: view_matrix[0][2],
    };
    let v_axis_y = FVector {
        x: view_matrix[1][0],
        y: view_matrix[1][1],
        z: view_matrix[1][2],
    };
    let v_axis_z = FVector {
        x: view_matrix[2][0],
        y: view_matrix[2][1],
        z: view_matrix[2][2],
    };

    let v_delta = position - pov.location;
    let v_transformed = FVector {
        x: v_delta.dot(&v_axis_y),
        y: v_delta.dot(&v_axis_z),
        z: v_delta.dot(&v_axis_x),
    };

    if (v_transformed.z < 1f64) {
        // return
        return Vector2 { x: 0.0, y: 0.0 };
    }

    let fov = pov.fov;
    let screen_center_x = 1920.0 / 2.0; // Replace 1920.0 with your actual screen width if needed
    let screen_center_y = 1080.0 / 2.0; // Replace 1080.0 with your actual screen height if needed
    let fov_rad = (fov as f32) * std::f32::consts::PI / 360.0;
    let tan_fov = fov_rad.tan();
    let screenpos = Vector2 {
        x: screen_center_x as f32
            + (v_transformed.x as f32) * (screen_center_x as f32 / tan_fov)
                / (v_transformed.z as f32),
        y: screen_center_y as f32
            - (v_transformed.y as f32) * (screen_center_x as f32 / tan_fov)
                / (v_transformed.z as f32),
    };

    return screenpos;
}
