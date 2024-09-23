use std::fmt;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Copy, Clone)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T>
where
    T: Copy + Default,
{
    pub fn new(x: T, y: T) -> Self {
        Vec2 { x, y }
    }
}

impl<T> Add for Vec2<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Vec2<T>;

    fn add(self, other: Vec2<T>) -> Vec2<T> {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T> Sub for Vec2<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vec2<T>;

    fn sub(self, other: Vec2<T>) -> Vec2<T> {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T> Mul<T> for Vec2<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec2<T>;

    fn mul(self, scalar: T) -> Vec2<T> {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl<T> fmt::Display for Vec2<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vec3<T>
where
    T: Copy + Default,
{
    pub fn new(x: T, y: T, z: T) -> Self {
        Vec3 { x, y, z }
    }
}

impl<T> Add for Vec3<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Vec3<T>;

    fn add(self, other: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl<T> Sub for Vec3<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vec3<T>;

    fn sub(self, other: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl<T> Mul<T> for Vec3<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec3<T>;

    fn mul(self, scalar: T) -> Vec3<T> {
        Vec3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Vec3<f32> {
    pub fn norm(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    #[allow(dead_code)]
    pub fn normalize(&mut self, l: f32) -> &Self {
        let length = self.norm();
        if length != 0.0 {
            let factor = l / length;
            self.x *= factor;
            self.y *= factor;
            self.z *= factor;
        }
        self
    }

    #[allow(dead_code)]
    pub fn to_i(&self) -> Vec3<i32> {
        Vec3 {
            x: self.x as i32,
            y: self.y as i32,
            z: self.z as i32,
        }
    }
}

impl Vec3<i32> {
    pub fn to_f(&self) -> Vec3<f32> {
        Vec3 {
            x: self.x as f32,
            y: self.y as f32,
            z: self.z as f32,
        }
    }
}

impl<T> Vec3<T>
where
    T: Mul<Output = T> + Add<Output = T> + Copy,
{
    #[allow(dead_code)]
    pub fn dot(self, other: Vec3<T>) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl<T> Vec3<T>
where
    T: Mul<Output = T> + Sub<Output = T> + Copy,
{
    #[allow(dead_code)]
    pub fn cross(self, other: Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl<T> fmt::Display for Vec3<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[allow(dead_code)]
pub type Vec2f = Vec2<f32>;
pub type Vec2i = Vec2<i32>;
pub type Vec3f = Vec3<f32>;
pub type Vec3i = Vec3<i32>;

pub struct Matrix {
    m: Vec<Vec<f32>>,
    rows: usize,
    cols: usize,
}

#[allow(dead_code)]
impl Matrix {
    fn new(r: usize, c: usize) -> Self {
        Matrix {
            m: vec![vec![0.0; c]; r],
            rows: r,
            cols: c,
        }
    }

    fn identity(dimensions: usize) -> Self {
        let mut e = Self::new(dimensions, dimensions);
        for i in 0..e.rows {
            for j in 0..e.cols {
                if i == j {
                    e.m[i][j] = 1.0;
                }
            }
        }
        e
    }

    fn nrows(&self) -> usize {
        self.rows
    }

    fn ncols(&self) -> usize {
        self.cols
    }

    fn transpose(&self) -> Self {
        let mut t = Matrix::new(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols {
                t.m[j][i] = self.m[i][j];
            }
        }
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(l: &Matrix, r: &Matrix) -> bool {
        if l.rows != r.rows || l.cols != r.cols {
            return false;
        }

        for i in 0..l.rows {
            for j in 0..l.cols {
                if (l.m[i][j] - r.m[i][j]).abs() > f32::EPSILON {
                    return false;
                }
            }
        }

        true
    }

    #[test]
    fn test_identity() {
        assert!(approx_eq(
            &Matrix::identity(2),
            &Matrix {
                m: vec![vec![1.0, 0.0], vec![0.0, 1.0]],
                rows: 2,
                cols: 2
            }
        ));
    }

    #[test]
    fn test_transpose() {
        let m = Matrix {
            m: vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]],
            rows: 2,
            cols: 3,
        };
        let mt = Matrix {
            m: vec![vec![1.0, 4.0], vec![2.0, 5.0], vec![3.0, 6.0]],
            rows: 3,
            cols: 2,
        };
        assert!(approx_eq(&m.transpose(), &mt));
    }
}
