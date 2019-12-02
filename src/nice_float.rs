use std::fmt;

use serde::de;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

type Float = f64;

const NAN: Float = std::f64::NAN;

#[derive(Debug)]
pub struct NiceFloat(Float);

impl From<&NiceFloat> for f64 {
    fn from(nf: &NiceFloat) -> Self {
        nf.0
    }
}

impl Serialize for NiceFloat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let x = &self.0;

        if x.is_nan() {
            serializer.serialize_str("NaN")
        } else {
            serializer.serialize_f64(*x)
        }
    }
}

impl<'de> Deserialize<'de> for NiceFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NiceFloatDeserializeVisitor;

        impl<'de> Visitor<'de> for NiceFloatDeserializeVisitor {
            type Value = NiceFloat;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a float or the string \"NaN\"")
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(NiceFloat(v as Float))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(NiceFloat(v as Float))
            }

            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(NiceFloat(v as Float))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(NiceFloat(v as Float))
            }

            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(NiceFloat(v as Float))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(NiceFloat(v as Float))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v == "NaN" {
                    Ok(NiceFloat(NAN))
                } else {
                    Err(E::invalid_value(de::Unexpected::Str(v), &self))
                }
            }
        }

        deserializer.deserialize_any(NiceFloatDeserializeVisitor)
    }
}
