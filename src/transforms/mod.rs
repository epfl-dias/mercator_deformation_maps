#[macro_use]
mod point;

pub mod affine;
pub mod gis;

use gis::K;

/*
enum Transform<'t> {
    Affine(AffineTransform),
    DeformationMap(GISDim, GISData<'t>),
}
*/
