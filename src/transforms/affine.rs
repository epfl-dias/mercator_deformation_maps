use arrayref::array_ref;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

// We tie this to GIS' restrictions for now.
use super::K;

pub struct AffineTransform {
    offsets: [f64; K],     // Expressed in millimeters
    matrix: [[f64; K]; K], // Rotation matrix
}

impl Debug for AffineTransform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AffineTransform {{ dimensions: {}x{}, offsets: {:?}, matrix: ",
            K, K, self.offsets
        )?;
        write!(f, "{:?} }}", self.matrix)
    }
}

impl AffineTransform {
    pub fn load_file(filename: &str) -> Result<Self, Box<dyn Error>> {
        let mut file_in = BufReader::new(File::open(&filename)?);

        let mut string = String::new();
        file_in.read_to_string(&mut string)?;

        let iter = string.lines();
        let offsets = &iter
            .clone()
            .take(1)
            .map(|line| {
                line.split_whitespace()
                    .map(|value| value.parse::<f64>().unwrap())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()[0];

        let matrix = iter
            .map(|line| {
                let v = line
                    .split_whitespace()
                    .map(|value| value.parse::<f64>().unwrap())
                    .collect::<Vec<_>>();

                *array_ref!(v, 0, K)
            })
            .collect::<Vec<_>>();

        Ok(Self {
            offsets: *array_ref!(offsets, 0, K),
            matrix: *array_ref!(matrix, 0, K),
        })
    }
}

pub fn load_file(basename: &str) -> Result<AffineTransform, Box<dyn Error>> {
    AffineTransform::load_file(basename)
}
