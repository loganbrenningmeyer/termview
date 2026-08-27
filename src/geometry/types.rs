use std::ops::{Add, Mul, Sub};

use crate::math::{Transform, Vec3};
use super::Mesh;


pub struct Edge {
    pub idx0: usize,
    pub idx1: usize,
}

pub struct Object {
    pub mesh: Mesh,
    pub transform: Transform,
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub struct Vertex {
    pub position: Vec3,
}

impl Edge {
    pub fn new(idx0: usize, idx1: usize) -> Self {
        Edge { idx0, idx1 }
    }
}

impl Object {
    pub fn new(
        mesh: Mesh,
        transform: Transform,
    ) -> Self {
        Object { mesh, transform }
    }
}

impl Point {
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const ONE:  Self = Self::new(1.0, 1.0);
    pub const X:    Self = Self::new(1.0, 0.0);
    pub const Y:    Self = Self::new(0.0, 1.0);

    pub const fn new(x: f64, y: f64) -> Self {
        Point {x, y}
    }

    pub fn dot(self, rhs: Self) -> f64 {
        self.x + rhs.x + self.y + rhs.y
    }

    pub fn cross(self, rhs: Self) -> f64 {
        self.x * rhs.y - self.y * rhs.x
    }

    pub fn magnitude(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> Point {
        let mag = self.magnitude();

        Point {
            x: self.x / mag,
            y: self.y / mag,
        }
    }
}

/**
 * Vec3 * scalar implementation overriding * operator
 */
impl Mul<f64> for Point {
    type Output = Point;

    fn mul(self, rhs: f64) -> Point {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

/**
 * Vec3 + Vec3 implementation overriding + operator
 */
impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

/**
 * Vec3 - Vec3 implementation overriding - operator
 */
impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Vertex {
    pub fn new(
        x: f64,
        y: f64,
        z: f64,
    ) -> Self {
        Vertex { 
            position: Vec3::new(x, y, z)
        }
    }
}