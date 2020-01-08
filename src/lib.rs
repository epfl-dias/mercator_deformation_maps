#[macro_use]
extern crate arrayref;

mod transforms;

pub use transforms::affine;
pub use transforms::gis;

#[cfg(test)]
#[macro_use]
extern crate measure_time;

#[cfg(test)]
mod nice_float;

#[cfg(test)]
mod tests;
