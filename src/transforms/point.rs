macro_rules! kd_point {
    ($point:ident, $output:ident, $dimensions:literal) => {
        #[derive(Clone, Debug)]
        pub struct $point(pub [$output; $dimensions]);

        impl $point {
            pub fn scale(&mut self, factor: $output) -> &mut Self {
                for k in &mut self.0 {
                    *k *= factor
                }

                self
            }

            pub fn dimensions(&self) -> usize {
                $dimensions
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
                for k in 0..$dimensions {
                    self.0[k] += rhs.0[k];
                }
            }
        }

        impl From<&Vec<$output>> for $point {
            fn from(v: &Vec<$output>) -> Self {
                Self(*array_ref!(v, 0, 3))
            }
        }

        impl From<Vec<$output>> for $point {
            fn from(v: Vec<$output>) -> Self {
                Self(*array_ref!(v, 0, 3))
            }
        }

        impl From<&$point> for Vec<$output> {
            fn from(p: &$point) -> Self {
                p.0[..].into()
            }
        }

        impl From<$point> for Vec<$output> {
            fn from(p: $point) -> Self {
                Self::from(&p)
            }
        }
    };
}

macro_rules! kd_point_is_nan {
    ($point:ident, $output:ident, $dimensions:literal) => {
        impl $point {
            pub fn is_nan(&self) -> bool {
                let mut bool = false;
                for k in 0..$dimensions {
                    bool = bool || self.0[k].is_nan();
                }

                bool
            }
        }
    };
}
