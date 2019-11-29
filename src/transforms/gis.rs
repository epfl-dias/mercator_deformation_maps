use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Read};
use std::{fmt, mem};

//use byteorder::BigEndian;
use byteorder::ByteOrder;
use byteorder::LittleEndian;
use memmap::Mmap;

use super::K;
use std::ops::{AddAssign, Index};

struct GISArrayData(Vec<f32>);

impl GISArrayData {
    fn load_file(basename: &str) -> Result<Self, Box<dyn Error>> {
        let filename = format!("{}.ima", basename);

        let file = File::open(&filename)?;
        let size = file.metadata()?.len() as usize;
        let count = size / mem::size_of::<f32>();

        debug_time!(
            "Loaded #{} {} GB {}\n\t> {} \n\t>",
            count,
            size as f32 / 2f32.powi(30),
            size,
            filename
        );

        // Load the data to memory
        let mut data = vec![0f32; count];

        let mmap = unsafe {
            Mmap::map(&file)
                .unwrap_or_else(|e| panic!("Unable to map in memory the file: {}: {}", filename, e))
        };
        LittleEndian::read_f32_into(mmap.as_ref(), &mut data);
        Ok(Self(data))
    }

    fn point3dd(&self, index: usize) -> Point3dd {
        let GISArrayData(data) = self;

        Point3dd([
            data[index * 3] as f64,
            data[index * 3 + 1] as f64,
            data[index * 3 + 2] as f64,
        ])
    }
}

impl Debug for GISArrayData {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

#[derive(Debug)]
enum ElementType {
    Point3Df,
}

#[derive(Debug)]
pub struct GISTransform {
    dimensions: Vec<usize>,
    element_type: ElementType,
    spacing: Vec<f64>, // Voxel spacing in millimeters, default is 1mm for unspecified values
    flat: Vec<bool>,
    data: GISArrayData,
}

impl GISTransform {
    pub fn load_file(basename: &str) -> Result<Self, Box<dyn Error>> {
        let filename = format!("{}.dim", basename);
        let mut file_in = BufReader::new(File::open(&filename)?);

        let mut string = String::new();
        file_in.read_to_string(&mut string)?;

        let mut iter = string.lines();
        let dimensions = iter
            .next()
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse::<usize>().unwrap())
            .collect::<Vec<_>>();

        let nb_dims = dimensions.len();

        let mut spacing = Vec::with_capacity(nb_dims);
        for _ in 0..nb_dims {
            spacing.push(1_f64); // Default value
        }

        for line in iter {
            let values = line.split_whitespace().collect::<Vec<_>>();

            for i in 0..(values.len() / 2) {
                let (param, value) = (values[i * 2], values[i * 2 + 1]);
                match param {
                    "-type" => assert_eq!(value, "POINT3DF"),
                    "-dx" => spacing[0] = value.parse::<f64>().unwrap(),
                    "-dy" => spacing[1] = value.parse::<f64>().unwrap(),
                    "-dz" => spacing[2] = value.parse::<f64>().unwrap(),
                    "-dt" => spacing[3] = value.parse::<f64>().unwrap(),
                    "-d4" => spacing[4] = value.parse::<f64>().unwrap(),
                    "-d5" => spacing[5] = value.parse::<f64>().unwrap(),
                    "-d6" => spacing[6] = value.parse::<f64>().unwrap(),
                    "-d7" => spacing[7] = value.parse::<f64>().unwrap(),
                    "-bo" => assert_eq!(value, "DCBA"),
                    "-om" => assert_eq!(value, "binar"),
                    _ => (),
                }
            }
        }

        let data = GISArrayData::load_file(&basename)?;

        Ok(Self {
            dimensions,
            element_type: ElementType::Point3Df,
            spacing,
            flat: vec![false, false, false],
            data,
        })
    }

    pub fn len(&self) -> usize {
        self.dimensions.iter().product()
    }

    pub fn dimensions(&self) -> &Vec<usize> {
        &self.dimensions
    }

    pub fn point3dd(&self, position: Vec<usize>) -> Point3dd {
        let mut index = 0;
        let mut stride = 1;

        for (i, v) in position.iter().enumerate() {
            index += stride * v;
            stride *= self.dimensions[i];
        }

        self.data.point3dd(index)
    }
}

// 2016: l now means long (64bit), i int (32bit), s short (16bit)
//type Point3dd = (f64, f64, f64);
//type Point3df = (f32, f32, f32);
//type Point3di = (i32, i32, i32);
//type Point3du = (usize, usize, usize);
#[derive(Debug)]
pub struct Point3dd(pub [f64; K]);
#[derive(Debug)]
pub struct Point3df(pub [f32; K]);
#[derive(Debug)]
struct Point3di(pub [i32; K]);
#[derive(Debug)]
struct Point3du(pub [usize; K]);

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
    };
}

macro_rules! indexed {
    ($point:ident, $output:ident) => {
        impl Index<usize> for $point {
            type Output = $output;

            fn index(&self, index: usize) -> &Self::Output {
                &self.0[index]
            }
        }
    };
}

point!(Point3dd, f64);

indexed!(Point3dd, f64);
indexed!(Point3df, f32);
indexed!(Point3di, i32);

impl AddAssign for Point3dd {
    fn add_assign(&mut self, rhs: Self) {
        for k in 0..K {
            self.0[k] += rhs.0[k];
        }
    }
}

impl From<&Vec<usize>> for Point3dd {
    fn from(v: &Vec<usize>) -> Self {
        Self([
            f64::from(v[0] as i32),
            f64::from(v[1] as i32),
            f64::from(v[2] as i32),
        ])
    }
}

impl From<&Vec<f64>> for Point3dd {
    fn from(v: &Vec<f64>) -> Self {
        Self([v[0], v[1], v[2]])
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

impl GISTransform {
    // Translated from C++ code from:
    //  https://github.com/brainvisa/aims-free/blob/master/aimsalgo/src/aimsalgo/registration/ffd.cc#L720-L805
    fn mm_to_spline_voxel(&self, p: &Point3dd) -> Point3dd {
        Point3dd([
            p[0] / self.spacing[0],
            p[1] / self.spacing[1],
            p[2] / self.spacing[2],
        ])
    }

    fn ctrl_point_delta(&self, i: i32, j: i32, k: i32) -> Point3dd {
        let d = self.point3dd(vec![i as usize, j as usize, k as usize]);
        trace!("ctrl_point_delta {:?}", d);
        d
    }

    // Input position in [mm] to deplacement in [mm]
    fn deformation_private(&self, p_image: &Point3dd) -> Point3dd {
        let p_spline = self.mm_to_spline_voxel(p_image);

        // FIXME: We should check the usize is >= 0 && <= i32::MAX per dimension
        let dim_d = Point3dd::from(self.dimensions());
        let dim = Point3di::from(self.dimensions());

        // Return NaN if the position is not covered by the deformation field
        // dim are integer values, so replace with converted version to f64
        if !(p_spline[0] >= 0.0
            && p_spline[0] < dim_d[0]
            && p_spline[1] >= 0.0
            && p_spline[1] < dim_d[1]
            && p_spline[2] >= 0.0
            && p_spline[2] < dim_d[2])
        {
            return Point3dd([std::f64::NAN; K]);
        }

        let k_spline = Point3di::from(&p_spline);
        let mut k_up = Point3di([k_spline[0] + 1, k_spline[1] + 1, k_spline[2] + 1]);

        let mut bt = [[0f64; 2], [0f64; 2], [0f64; 2]];
        for k in 0..K {
            if self.flat[k] || k_up[k] >= dim[k] {
                k_up.0[k] = k_spline[k];
                bt[k][0] = 1.0;
                bt[k][1] = 0.0;
            } else {
                bt[k][1] = p_spline[k] - f64::from(k_spline[k]);
                bt[k][0] = 1.0 - bt[k][1];
            }
        }

        let mut deformation = Point3dd([0., 0., 0.]);
        for k in k_spline[2]..=k_up[2] {
            for j in k_spline[1]..=k_up[1] {
                for i in k_spline[0]..=k_up[0] {
                    let mut p = self.ctrl_point_delta(i, j, k);
                    p.scale(bt[0][(i - k_spline[0]) as usize])
                        .scale(bt[1][(j - k_spline[1]) as usize])
                        .scale(bt[2][(k - k_spline[2]) as usize]);

                    deformation += p;
                }
            }
        }
        trace!("deformation {:?}", deformation);

        deformation
    }

    pub fn deformation(&self, p: &Point3dd) -> Point3dd {
        let mut t = Point3dd([f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]);

        t += self.deformation_private(&t);

        t
    }
}

pub fn load_file(basename: &str) -> Result<GISTransform, Box<dyn Error>> {
    GISTransform::load_file(basename)
}
