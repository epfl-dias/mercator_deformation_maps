use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::mem;
use std::ops::AddAssign;
use std::ops::Index;

//use byteorder::BigEndian;
use byteorder::ByteOrder;
use byteorder::LittleEndian;
use memmap::Mmap;

use super::point::*;
use super::K;

struct GISArrayData(Vec<f32>);

impl GISArrayData {
    fn load_file(basename: &str) -> Result<Self, Box<dyn Error>> {
        let filename = format!("{}.ima", basename);

        let file = File::open(&filename)?;
        let size = file.metadata()?.len() as usize;
        let count = size / mem::size_of::<f32>();

        #[cfg(test)]
        info_time!(
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

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn point3dd(&self, index: usize) -> Point3dd {
        let GISArrayData(data) = self;

        Point3dd([
            data[index * 3] as f64,
            data[index * 3 + 1] as f64,
            data[index * 3 + 2] as f64,
        ])
    }

    fn point3df(&self, index: usize) -> Point3df {
        let GISArrayData(data) = self;

        Point3df([
            data[index * 3] as f32,
            data[index * 3 + 1] as f32,
            data[index * 3 + 2] as f32,
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

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn dimensions(&self) -> &Vec<usize> {
        &self.dimensions
    }

    pub fn dimensions_mm(&self) -> Point3dd {
        let d = &self.dimensions;
        let s = &self.spacing;

        Point3dd([d[0] as f64 * s[0], d[1] as f64 * s[1], d[2] as f64 * s[2]])
    }

    fn index(&self, position: Vec<usize>) -> usize {
        let mut index = 0;
        let mut stride = 1;

        for (i, v) in position.iter().enumerate() {
            index += stride * v;
            stride *= self.dimensions[i];
        }

        index
    }

    pub fn point3dd(&self, position: Vec<usize>) -> Point3dd {
        self.data.point3dd(self.index(position))
    }

    pub fn point3df(&self, position: Vec<usize>) -> Point3df {
        self.data.point3df(self.index(position))
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
        self.point3dd(vec![i as usize, j as usize, k as usize])
    }

    // Input position in [mm] to displacement in [mm]
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
            //warn!("Returning NaN as we are outside of the deformation field!");
            warn!("p_spline {:?}, dim_d {:?}", p_spline, dim_d);
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
                    /*if p.is_nan() {
                        trace!("ctrl_point_delta: [{}, {}, {}] p {:?}", i, j, k, p);
                    }*/
                    p.scale(bt[0][(i - k_spline[0]) as usize])
                        .scale(bt[1][(j - k_spline[1]) as usize])
                        .scale(bt[2][(k - k_spline[2]) as usize]);
                    /*
                    if p.is_nan() {
                        trace!("scaling failed: [{}, {}, {}] p {:?}", i, j, k, p);
                    }*/

                    deformation += p;
                }
            }
        }
        if deformation.is_nan() {
            warn!("deformation {:?} -> {:?}", p_image, deformation);
        } /*else {
              trace!("deformation {:?} -> {:?}", p_image, deformation);
          }*/

        deformation
    }

    pub fn deformation(&self, p: &Point3dd) -> Point3dd {
        let mut t = p.clone();

        t += self.deformation_private(&t);

        t
    }
}

pub fn load_file(basename: &str) -> Result<GISTransform, Box<dyn Error>> {
    GISTransform::load_file(basename)
}
