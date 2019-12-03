use std::ops::AddAssign;
use std::ops::Index;

use super::K;

// 2016: l now means long (64bit), i int (32bit), s short (16bit)
#[derive(Clone, Debug)]
pub struct Point3dd(pub [f64; K]);
#[derive(Clone, Debug)]
pub struct Point3df(pub [f32; K]);
#[derive(Clone, Debug)]
pub struct Point3di(pub [i32; K]);

macro_rules! point {
    ($point:ident, $output:ident) => {
        impl $point {
            pub fn scale(&mut self, factor: $output) -> &mut Self {
                for k in &mut self.0 {
                    *k *= factor
                }

                self
            }
        }

        impl Index<usize> for $point {
            type Output = $output;

            fn index(&self, index: usize) -> &Self::Output {
                &self.0[index]
            }
        }

        impl AddAssign for $point {
            fn add_assign(&mut self, rhs: Self) {
                for k in 0..K {
                    self.0[k] += rhs.0[k];
                }
            }
        }

        impl From<&Vec<$output>> for $point {
            fn from(v: &Vec<$output>) -> Self {
                Self([v[0], v[1], v[2]])
            }
        }

        impl From<Vec<$output>> for $point {
            fn from(v: Vec<$output>) -> Self {
                $point::from(&v)
            }
        }

        impl From<&$point> for Vec<$output> {
            fn from(p: &$point) -> Self {
                vec![p[0], p[1], p[2]]
            }
        }

        impl From<$point> for Vec<$output> {
            fn from(p: $point) -> Self {
                Self::from(&p)
            }
        }
    };
}

point!(Point3dd, f64);
point!(Point3df, f32);
point!(Point3di, i32);

macro_rules! is_nan {
    ($point:ident, $output:ident) => {
        impl $point {
            pub fn is_nan(&self) -> bool {
                let mut bool = false;
                for k in 0..K {
                    bool = bool || self.0[k].is_nan();
                }

                bool
            }
        }
    };
}

is_nan!(Point3dd, f64);
is_nan!(Point3df, f32);

impl From<&Vec<usize>> for Point3dd {
    fn from(v: &Vec<usize>) -> Self {
        Self([
            f64::from(v[0] as i32),
            f64::from(v[1] as i32),
            f64::from(v[2] as i32),
        ])
    }
}

impl From<&Vec<usize>> for Point3di {
    fn from(v: &Vec<usize>) -> Self {
        Self([v[0] as i32, v[1] as i32, v[2] as i32])
    }
}

impl From<&Point3dd> for Point3di {
    fn from(p: &Point3dd) -> Self {
        Point3di([
            p[0].floor() as i32,
            p[1].floor() as i32,
            p[2].floor() as i32,
        ])
    }
}

/*
Is it better than the above?

#[derive(Clone, Debug)]
pub struct Pointkd<T>(pub [T; K]);

impl<T> Pointkd<T>
where
    T: std::ops::MulAssign + Copy,
{
    pub fn scale(&mut self, factor: T) -> &mut Self {
        for k in &mut self.0 {
            *k *= factor
        }

        self
    }
}

impl<T> Index<usize> for Pointkd<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> AddAssign for Pointkd<T>
where
    T: std::ops::AddAssign + Copy,
{
    fn add_assign(&mut self, rhs: Self) {
        for k in 0..K {
            self.0[k] += rhs.0[k];
        }
    }
}

impl<T> From<&Vec<T>> for Pointkd<T>
where
    T: Copy,
{
    fn from(v: &Vec<T>) -> Self {
        Self([v[0], v[1], v[2]])
    }
}

impl<T> From<Vec<T>> for Pointkd<T>
where
    T: Copy,
{
    fn from(v: Vec<T>) -> Self {
        Self::from(&v)
    }
}

macro_rules! pointkd {
    ($output:ident) => {
        impl From<&Pointkd<$output>> for Vec<$output> {
            fn from(p: &Pointkd<$output>) -> Self {
                vec![p[0], p[1], p[2]]
            }
        }

        impl From<Pointkd<$output>> for Vec<$output> {
            fn from(p: Pointkd<$output>) -> Self {
                Self::from(&p)
            }
        }
    };
}

pointkd!(f64);
pointkd!(f32);
pointkd!(i32);

macro_rules! is_nan_kd {
    ($output:ident) => {
        impl Pointkd<$output> {
            pub fn is_nan(&self) -> bool {
                let mut bool = false;
                for k in 0..K {
                    bool = bool || self.0[k].is_nan();
                }

                bool
            }
        }
    };
}

is_nan_kd!(f32);
is_nan_kd!(f64);
*/
