use core::ops::Mul;
use std::ops::Add;


#[derive(Debug, Default, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub const fn mat2(x_axis: Vec2, y_axis: Vec2) -> Mat2 {
    Mat2::from_cols(x_axis, y_axis)
}

pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub const fn from_array(a: [f32; 2]) -> Self {
        Self::new(a[0], a[1])
    }
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }
    pub const ZERO: Self = Self::splat(0.0);
    pub const X: Self = Self::new(1.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0);
}

impl Add<Vec2> for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x.add(rhs.x),
            y: self.y.add(rhs.y),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Mat2 {
    pub x_axis: Vec2,
    pub y_axis: Vec2,
}

impl Mat2 {
    const fn new(m00: f32, m01: f32, m10: f32, m11: f32) -> Self {
        Self {
            x_axis: Vec2::new(m00, m01),
            y_axis: Vec2::new(m10, m11),
        }
    }
    pub const ZERO: Self = Self::from_cols(Vec2::ZERO, Vec2::ZERO);
    
    pub const fn from_cols(x_axis: Vec2, y_axis: Vec2) -> Self {
        Self { x_axis, y_axis }
    }

    pub fn mul_vec2(&self, rhs: Vec2) -> Vec2 {
        #[allow(clippy::suspicious_operation_groupings)]
        Vec2::new(
            (self.x_axis.x * rhs.x) + (self.y_axis.x * rhs.y),
            (self.x_axis.y * rhs.x) + (self.y_axis.y * rhs.y),
        )
    }
    pub const fn from_cols_array(m: &[f32; 4]) -> Self {
        Self::new(m[0], m[1], m[2], m[3])
    }
    pub fn mul_mat2(&self, rhs: &Self) -> Self {
        Self::from_cols(self.mul(rhs.x_axis), self.mul(rhs.y_axis))
    }
}

impl Mul<Vec2> for Mat2 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Self::Output {
        self.mul_vec2(rhs)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Affine2 {
    pub matrix2: Mat2,
    pub translation: Vec2,
}

impl Affine2 {
    pub const fn from_cols_array(m: &[f32; 6]) -> Self {
        Self {
            matrix2: Mat2::from_cols_array(&[m[0], m[1], m[2], m[3]]),
            translation: Vec2::from_array([m[4], m[5]]),
        }
    }
    pub fn transform_point2(&self, rhs: Vec2) -> Vec2 {
        self.matrix2 * rhs + self.translation
    }
}

impl Mul<Mat2> for Mat2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_mat2(&rhs)
    }
}

impl Mul for Affine2 {
    type Output = Affine2;

    #[inline]
    fn mul(self, rhs: Affine2) -> Self::Output {
        Self {
            matrix2: self.matrix2 * rhs.matrix2,
            translation: self.matrix2 * rhs.translation + self.translation,
        }
    }
}

pub(crate) fn ddot(coeffs: &[f32], params: &[f32], np: usize) -> f32 {
    let mut total = 0.0;
    for idx in 0..np {
        total += coeffs[idx] * params[idx];
    }
    total += coeffs[np];
    total
}

pub(crate) fn fill_vector(coeffs: &[f32], params: &[f32], np: usize, v: &mut Vec2) {
    v.x = ddot(coeffs, params, np);
    v.y = ddot(&coeffs[(np + 1)..], params, np);
}

pub(crate) fn fill_affine(coeffs: &[f32], params: &[f32], np: usize, m: &mut Affine2) {
    m.matrix2.x_axis = Vec2::from_array([ddot(coeffs, params, np), ddot(&coeffs[(np * 3 + 3)..], params, np)]);
    m.matrix2.y_axis = Vec2::from_array([ddot(&coeffs[(np + 1)..], params, np), ddot(&coeffs[(np * 4 + 4)..], params, np)]);
    m.translation = Vec2::from_array([ddot(&coeffs[(np * 2 + 2)..], params, np), ddot(&coeffs[(np * 5 +5 )..], params, np)]);
}

pub(crate) fn r_match(p: &Vec2, q: &Vec2) -> Affine2 {
    Affine2::from_cols_array(&[q.x - p.x, q.y - p.y, p.y - q.y, q.x - p.x,p.x, p.y])
}

pub(crate) const M_ORIENTS: [Affine2; 4] = [
    Affine2::from_cols_array(&[1.0, 0.0, 0.0, 1.0, 0.0, 0.0]), // IDENTITY
    Affine2::from_cols_array(&[-1.0, 0.0, 0.0, -1.0, 1.0, 0.0]), // ROT
    Affine2::from_cols_array(&[-1.0, 0.0, 0.0, 1.0, 1.0, 0.0]), // FLIP
    Affine2::from_cols_array(&[1.0, 0.0, 0.0, -1.0, 0.0, 0.0]), // ROFL
];

pub(crate) static TSPI_U: [Affine2; 2] = [
    Affine2::from_cols_array(&[0.5, 0.0, 0.0, 0.5, 0.0, 0.0]),
    Affine2::from_cols_array(&[-0.5, 0.0, 0.0, 0.5, 1.0, 0.0]),
];

pub(crate) static TSPI_S: [Affine2; 2] = [
    Affine2::from_cols_array(&[0.5, 0.0, 0.0, 0.5, 0.0, 0.0]),
    Affine2::from_cols_array(&[-0.5, 0.0, 0.0, -0.5, 0.0, 0.0]),
];


