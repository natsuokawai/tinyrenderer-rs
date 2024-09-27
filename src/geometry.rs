use std::fmt::{self, Formatter};
use std::ops::{Add, Index, IndexMut, Mul, Sub};

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
    pub fn new(r: usize, c: usize) -> Self {
        Matrix {
            m: vec![vec![0.0; c]; r],
            rows: r,
            cols: c,
        }
    }

    pub fn identity(dimensions: usize) -> Self {
        let mut e = Self::new(dimensions, dimensions);
        for i in 0..e.rows {
            for j in 0..e.cols {
                if i == j {
                    e[i][j] = 1.0;
                }
            }
        }
        e
    }

    pub fn projection(z: f32) -> Self {
        let mut mat = Matrix::identity(4);
        mat[3][2] = -1.0 / z;
        mat
    }

    pub fn nrows(&self) -> usize {
        self.rows
    }

    pub fn ncols(&self) -> usize {
        self.cols
    }

    pub fn transpose(&self) -> Self {
        let mut t = Matrix::new(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols {
                t[j][i] = self[i][j];
            }
        }
        t
    }

    pub fn inverse(&self) -> Option<Self> {
        if self.rows != self.cols {
            return None;
        }

        let n = self.rows;
        let (l, u) = self.lu_decompose();
        let mut inverse = Matrix::new(n, n);

        for i in 0..n {
            // step 1: solve L * y = e
            let mut y = vec![0.0; n];
            for j in 0..n {
                let mut sum = 0.0;
                for k in 0..j {
                    sum += l[j][k] * y[k];
                }
                y[j] = if i == j { 1.0 } else { 0.0 } - sum;
            }

            // step 2: solve U * x = y
            let mut x = vec![0.0; n];
            for j in (0..n).rev() {
                let mut sum = 0.0;
                for k in j + 1..n {
                    sum += u[j][k] * x[k];
                }
                x[j] = (y[j] - sum) / u[j][j];
            }

            for j in 0..n {
                inverse[j][i] = x[j];
            }
        }

        Some(inverse)
    }

    fn lu_decompose(&self) -> (Matrix, Matrix) {
        let n = self.rows;
        let mut l = Matrix::identity(n);
        let mut u = Matrix::new(n, n);

        for i in 0..n {
            for k in i..n {
                let mut sum = 0.0;
                for j in 0..i {
                    sum += l[i][j] * u[j][k];
                }
                u[i][k] = self[i][k] - sum;
            }

            for k in i + 1..n {
                let mut sum = 0.0;
                for j in 0..i {
                    sum += l[k][j] * u[j][i];
                }
                l[k][i] = (self[k][i] - sum) / u[i][i];
            }
        }

        (l, u)
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut s = String::new();
        for i in 0..self.rows {
            for j in 0..self.cols {
                s.push_str(&format!("{:.2} ", self[i][j]));
                if j < self.cols - 1 {
                    s.push_str("\t");
                }
            }
            s.push('\n');
        }
        write!(f, "{}", s)
    }
}

impl Index<usize> for Matrix {
    type Output = Vec<f32>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.m[index]
    }
}

impl IndexMut<usize> for Matrix {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.m[index]
    }
}

impl Mul for Matrix {
    type Output = Matrix;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut res = Self::new(self.rows, rhs.cols);

        for i in 0..self.rows {
            for k in 0..rhs.cols {
                for j in 0..self.cols {
                    res[i][k] += self[i][j] * rhs[j][k];
                }
            }
        }

        res
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
                if (l[i][j] - r[i][j]).abs() > f32::EPSILON {
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

    #[test]
    fn test_mul() {
        let a = Matrix {
            m: vec![vec![1.0, 2.0], vec![3.0, 1.0], vec![3.0, 2.0]],
            rows: 3,
            cols: 2,
        };
        let b = Matrix {
            m: vec![vec![1.0, 3.0, 5.0], vec![2.0, 4.0, 1.0]],
            rows: 2,
            cols: 3,
        };
        let c = Matrix {
            m: vec![
                vec![5.0, 11.0, 7.0],
                vec![5.0, 13.0, 16.0],
                vec![7.0, 17.0, 17.0],
            ],
            rows: 3,
            cols: 3,
        };
        assert!(approx_eq(&(a * b), &c));
    }

    #[test]
    fn test_inverse() {
        let m = Matrix {
            m: vec![
                vec![1.0, 1.0, -1.0],
                vec![-2.0, 0.0, 1.0],
                vec![0.0, 2.0, 1.0],
            ],
            rows: 3,
            cols: 3,
        };
        let mi = Matrix {
            m: vec![
                vec![-0.5, -0.75, 0.25],
                vec![0.5, 0.25, 0.25],
                vec![-1.0, -0.5, 0.5],
            ],
            rows: 3,
            cols: 3,
        };
        assert!(match m.inverse() {
            Some(m_inverse) => approx_eq(&m_inverse, &mi),
            None => false,
        });
    }
}
