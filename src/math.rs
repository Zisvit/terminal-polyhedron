//! 3D mathematics: vectors, rotation matrices, perspective projection.

use std::f64::consts::PI;
use std::ops::Sub;

/// A 3D vector / point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Vec3 {
        self.sub(rhs)
    }
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub fn mul(self, s: f64) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }

    pub fn dot(self, other: Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        if len == 0.0 {
            return Vec3::ZERO;
        }
        self.mul(1.0 / len)
    }
}

/// Rotate a vector around the X axis by `angle` radians.
pub fn rotate_x(v: Vec3, angle: f64) -> Vec3 {
    let (sin, cos) = angle.sin_cos();
    Vec3::new(v.x, v.y * cos - v.z * sin, v.y * sin + v.z * cos)
}

/// Rotate a vector around the Y axis by `angle` radians.
pub fn rotate_y(v: Vec3, angle: f64) -> Vec3 {
    let (sin, cos) = angle.sin_cos();
    Vec3::new(v.x * cos + v.z * sin, v.y, -v.x * sin + v.z * cos)
}

/// Rotate a vector around the Z axis by `angle` radians.
pub fn rotate_z(v: Vec3, angle: f64) -> Vec3 {
    let (sin, cos) = angle.sin_cos();
    Vec3::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos, v.z)
}

/// Apply rotation around X, then Y, then Z.
pub fn rotate(v: Vec3, angle_x: f64, angle_y: f64, angle_z: f64) -> Vec3 {
    let v = rotate_x(v, angle_x);
    let v = rotate_y(v, angle_y);
    rotate_z(v, angle_z)
}

/// Perspective project a 3D point into 2D screen coordinates.
///
/// `focal` — focal length (distance from camera to projection plane).
/// `distance` — distance from camera to the object along Z.
/// Returns (screen_x, screen_y, projected_z) where projected_z is the depth
/// after projection (used for sorting/shading).
pub fn project(v: Vec3, focal: f64, distance: f64) -> (f64, f64, f64) {
    let z = v.z + distance;
    let factor = focal / z;
    (v.x * factor, v.y * factor, z)
}

pub fn deg_to_rad(deg: f64) -> f64 {
    deg * PI / 180.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_x_90_degrees_maps_y_to_z() {
        let v = Vec3::new(0.0, 1.0, 0.0);
        let r = rotate_x(v, deg_to_rad(90.0));
        assert!((r.y - 0.0).abs() < 1e-10);
        assert!((r.z - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rotate_y_90_degrees_maps_x_to_neg_z() {
        let v = Vec3::new(1.0, 0.0, 0.0);
        let r = rotate_y(v, deg_to_rad(90.0));
        assert!((r.x - 0.0).abs() < 1e-10);
        assert!((r.z - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn project_preserves_relative_depth() {
        let near = Vec3::new(0.0, 0.0, 1.0);
        let far = Vec3::new(0.0, 0.0, 5.0);
        let (_, _, zn) = project(near, 1.0, 5.0);
        let (_, _, zf) = project(far, 1.0, 5.0);
        assert!(zn < zf);
    }
}
