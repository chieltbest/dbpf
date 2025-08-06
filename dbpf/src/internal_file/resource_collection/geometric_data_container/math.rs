use crate::binrw;
use std::ops::{Add, AddAssign, Mul};

#[binrw]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub fn identity() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }

    pub fn inverse(self) -> Self {
        let sq_mag = self.x*self.x + self.y*self.y + self.z*self.z + self.w*self.w;
        Self {
            x: -self.x / sq_mag,
            y: -self.y / sq_mag,
            z: -self.z / sq_mag,
            w: self.w / sq_mag,
        }
    }
}

#[binrw]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vertex {
    pub fn identity() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn inverse(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Add for Vertex {
    type Output = Vertex;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Vertex {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[binrw]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Transform {
    pub rotation: Quaternion,
    pub translation: Vertex,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            translation: Vertex::identity(),
            rotation: Quaternion::identity(),
        }
    }

    pub fn inverse(&self) -> Self {
        let quat_inverse = self.rotation.inverse();
        Self {
            translation: Mat4::rotation(quat_inverse) * self.translation.inverse(),
            rotation: quat_inverse,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Mat4(pub [f32; 16]);

impl Mat4 {
    pub fn identity() -> Self {
        Self([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    pub fn translation(v: Vertex) -> Self {
        Self([
            1.0, 0.0, 0.0, v.x,
            0.0, 1.0, 0.0, v.y,
            0.0, 0.0, 1.0, v.z,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    pub fn rotation(q: Quaternion) -> Self {
        // https://www.songho.ca/opengl/gl_quaternion.html
        let Quaternion { x, y, z, w } = q;
        Self([
            1.0 - 2.0*y*y - 2.0*z*z, 2.0*x*y - 2.0*w*z,     2.0*x*z + 2.0*w*y,       0.0,
            2.0*x*y + 2.0*w*z,       1.0 - 2.0*x*x - 2.0*z*z, 2.0*y*z - 2.0*w*x,       0.0,
            2.0*x*z - 2.0*w*y,       2.0*y*z + 2.0*w*x,     1.0 - 2.0*x*x - 2.0*y*y, 0.0,
            0.0,                     0.0,                   0.0,                     1.0,
        ])
    }

    pub fn transform(t: Transform) -> Self {
        Self::translation(t.translation) * Self::rotation(t.rotation)
    }

    pub fn rotation_x(a: f32) -> Self {
        Self([
            1.0, 0.0,      0.0,     0.0,
            0.0, a.cos(), -a.sin(), 0.0,
            0.0, a.sin(),  a.cos(), 0.0,
            0.0, 0.0,      0.0,     1.0,
        ])
    }

    pub fn rotation_y(a: f32) -> Self {
        Self([
            a.cos(), 0.0, -a.sin(), 0.0,
            0.0,     1.0,  0.0,     0.0,
            a.sin(), 0.0,  a.cos(), 0.0,
            0.0,     0.0,  0.0,     1.0,
        ])
    }

    pub fn rotation_z(a: f32) -> Self {
        Self([
            a.cos(), -a.sin(), 0.0, 0.0,
            a.sin(),  a.cos(), 0.0, 0.0,
            0.0,      0.0,     1.0, 0.0,
            0.0,      0.0,     0.0, 1.0,
        ])
    }

    pub fn projection(near_clip: f32, screen_ratio: f32) -> Self {
        let sr_sqrt = screen_ratio.sqrt();
        Self([
            sr_sqrt, 0.0, 0.0, 0.0,
            0.0, 1.0 / sr_sqrt, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 1.0, near_clip,
        ])
    }

    pub fn swap_axes(self, ax0: usize, ax1: usize) -> Self {
        let mut new = self;
        for (l, r) in (ax0*4..ax0*4+4).zip(ax1*4..ax1*4+4) {
            new.0.swap(l, r);
        }
        new
    }

    pub fn transpose(self) -> Self {
        Self([
            self.0[0], self.0[4], self.0[8], self.0[12],
            self.0[1], self.0[5], self.0[9], self.0[13],
            self.0[2], self.0[6], self.0[10], self.0[14],
            self.0[3], self.0[7], self.0[11], self.0[15],
        ])
    }
}

impl Mul<Mat4> for Mat4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let l = self.0;
        let r = rhs.0;
        let new_slice: Vec<_> = (0..4).flat_map(|y| {
            (0..4).map(move |x| {
                (0..4).map(move |z| {
                    l[z + y*4] * r[x + z*4]
                }).sum()
            })
        }).collect();
        Self(new_slice[0..16].try_into().unwrap())
    }
}

impl Mul<Vertex> for Mat4 {
    type Output = Vertex;

    fn mul(self, rhs: Vertex) -> Self::Output {
        let v = [rhs.x, rhs.y, rhs.z, 1.0];
        let out: Vec<_> = (0..4).map(move |y| {
            (0..4).map(move |x| {
                self.0[x + y*4] * v[x]
            }).sum()
        }).collect();
        Vertex {
            x: out[0],
            y: out[1],
            z: out[2],
        }
    }
}