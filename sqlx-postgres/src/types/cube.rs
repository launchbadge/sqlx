use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::types::Type;
use crate::{PgArgumentBuffer, PgTypeInfo, PgValueRef, Postgres};

#[derive(Debug, Clone, PartialEq)]
pub enum PgCube {
    Point(f64),
    ZeroVolume(Vec<f64>),
    OneDimensionInterval(f64, f64),
    MultiDimension(Vec<Vec<f64>>),
}

impl Type<Postgres> for PgCube {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("cube")
    }
}

impl<'r> Decode<'r, Postgres> for PgCube {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = value.as_bytes()?;
        Ok(PgCube::deserialize(bytes)?)
    }
}

impl<'q> Encode<'q, Postgres> for PgCube {
    fn produces(&self) -> Option<PgTypeInfo> {
        Some(PgTypeInfo::with_name("cube"))
    }

    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend_from_slice(&self.serialize());

        IsNull::No
    }
}

const BYTE_WIDTH: usize = 8;

impl PgCube {
    fn serialize(&self) -> Vec<u8> {
        let mut buff: Vec<u8> = vec![];
        match self {
            PgCube::Point(value) => {
                buff.extend_from_slice(&[128, 0, 0, 1]);
                buff.extend_from_slice(&value.to_be_bytes());
            }
            PgCube::ZeroVolume(values) => {
                let dimension = values.len() as u8;
                buff.extend_from_slice(&[128, 0, 0]);
                buff.extend_from_slice(&dimension.to_be_bytes());
                let bytes = values
                    .into_iter()
                    .flat_map(|v| v.to_be_bytes())
                    .collect::<Vec<u8>>();
                buff.extend_from_slice(&bytes);
            }
            PgCube::OneDimensionInterval(x, y) => {
                buff.extend_from_slice(&[0, 0, 0, 1]);
                buff.extend_from_slice(&x.to_be_bytes());
                buff.extend_from_slice(&y.to_be_bytes());
            }
            PgCube::MultiDimension(multi_values) => {
                let dimension = multi_values
                    .first()
                    .map(|arr| arr.len() as u8)
                    .unwrap_or(1 as u8);
                buff.extend_from_slice(&[0, 0, 0]);
                buff.extend_from_slice(&dimension.to_be_bytes());
                let bytes = multi_values
                    .into_iter()
                    .flat_map(|inner| inner)
                    .flat_map(|v| v.to_be_bytes())
                    .collect::<Vec<u8>>();
                buff.extend_from_slice(&bytes);
            }
        };
        buff
    }

    fn get_f64_from_bytes(bytes: &[u8], start: usize) -> Result<f64, sqlx_core::Error> {
        bytes[start..start + BYTE_WIDTH]
            .try_into()
            .map(f64::from_be_bytes)
            .map_err(|err| {
                sqlx_core::Error::Decode(format!("Invalid bytes slice: {:?}", err).into())
            })
    }
    // Helper to deserialize a vector of f64 values
    fn deserialize_vector(bytes: &[u8], start_index: usize) -> Result<Vec<f64>, sqlx_core::Error> {
        let steps = (bytes.len() - start_index) / BYTE_WIDTH;
        (0..steps)
            .map(|i| Self::get_f64_from_bytes(&bytes, start_index + i * BYTE_WIDTH))
            .collect()
    }

    // Helper to deserialize a matrix of f64 values
    fn deserialize_matrix(
        bytes: &[u8],
        start_index: usize,
        dim: usize,
    ) -> Result<Vec<Vec<f64>>, sqlx_core::Error> {
        let step = BYTE_WIDTH * dim;
        let steps = (bytes.len() - start_index) / step;

        (0..steps)
            .map(|step_idx| {
                (0..dim)
                    .map(|dim_idx| {
                        Self::get_f64_from_bytes(
                            &bytes,
                            start_index + step_idx * step + dim_idx * BYTE_WIDTH,
                        )
                    })
                    .collect()
            })
            .collect()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, sqlx_core::Error> {
        let cube_type = bytes[0] as usize;
        let dimensionality = bytes[3] as usize;
        let start_index = 4;

        match (cube_type, dimensionality) {
            (128, 1) => {
                let point = Self::get_f64_from_bytes(&bytes, 4)?;
                Ok(PgCube::Point(point))
            }
            (128, _) => Ok(PgCube::ZeroVolume(Self::deserialize_vector(
                &bytes,
                start_index,
            )?)),
            (0, 1) => {
                let x_start = 4; // 16 bytes per dimension (2 coordinates)
                let y_start = x_start + BYTE_WIDTH; // Upper right follows lower left
                let x = Self::get_f64_from_bytes(&bytes, x_start)?;
                let y = Self::get_f64_from_bytes(&bytes, y_start)?;
                Ok(PgCube::OneDimensionInterval(x, y))
            }
            (0, dim) => Ok(PgCube::MultiDimension(Self::deserialize_matrix(
                &bytes,
                start_index,
                dim,
            )?)),
            (flag, dimension) => Err(sqlx_core::Error::Decode(
                format!(
                    "Could not deserialise cube with flag {} and dimension {}. Bytes: {:?}",
                    flag, dimension, bytes
                )
                .into(),
            )),
        }
    }
}

#[cfg(test)]
mod cube_tests {
    use crate::types::PgCube;

    const POINT_BYTES: &[u8] = &[128, 0, 0, 1, 64, 0, 0, 0, 0, 0, 0, 0];
    const ZERO_VOLUME_BYTES: &[u8] = &[
        128, 0, 0, 2, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0,
    ];
    const ONE_DIMENSIONAL_INTERVAL_BYTES: &[u8] = &[
        0, 0, 0, 1, 64, 28, 0, 0, 0, 0, 0, 0, 64, 32, 0, 0, 0, 0, 0, 0,
    ];
    const MULTI_DIMENSION_2_DIM_BYTES: &[u8] = &[
        0, 0, 0, 2, 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0,
        64, 16, 0, 0, 0, 0, 0, 0,
    ];
    const MULTI_DIMENSION_3_DIM_BYTES: &[u8] = &[
        0, 0, 0, 3, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0, 64, 16, 0, 0, 0, 0, 0, 0, 64,
        20, 0, 0, 0, 0, 0, 0, 64, 24, 0, 0, 0, 0, 0, 0, 64, 28, 0, 0, 0, 0, 0, 0,
    ];

    #[test]
    fn can_deserialise_point_type() {
        let cube = PgCube::deserialize(POINT_BYTES).unwrap();
        assert_eq!(cube, PgCube::Point(2.))
    }

    #[test]
    fn can_serialise_point_type() {
        assert_eq!(PgCube::Point(2.).serialize(), POINT_BYTES,)
    }
    #[test]
    fn can_deserialise_zero_volume() {
        let cube = PgCube::deserialize(ZERO_VOLUME_BYTES).unwrap();
        assert_eq!(cube, PgCube::ZeroVolume(vec![2., 3.]));
    }

    #[test]
    fn can_serialise_zero_volume() {
        assert_eq!(
            PgCube::ZeroVolume(vec![2., 3.]).serialize(),
            ZERO_VOLUME_BYTES
        );
    }

    #[test]
    fn can_deserialise_one_dimension_interval() {
        let cube = PgCube::deserialize(ONE_DIMENSIONAL_INTERVAL_BYTES).unwrap();
        assert_eq!(cube, PgCube::OneDimensionInterval(7., 8.))
    }
    #[test]
    fn can_serialise_one_dimension_interval() {
        assert_eq!(
            PgCube::OneDimensionInterval(7., 8.).serialize(),
            ONE_DIMENSIONAL_INTERVAL_BYTES
        )
    }
    #[test]
    fn can_deserialise_multi_dimension_2_dimension() {
        let cube = PgCube::deserialize(MULTI_DIMENSION_2_DIM_BYTES).unwrap();
        assert_eq!(
            cube,
            PgCube::MultiDimension(vec![vec![1., 2.], vec![3., 4.]])
        )
    }

    #[test]
    fn can_serialise_multi_dimension_2_dimension() {
        assert_eq!(
            PgCube::MultiDimension(vec![vec![1., 2.], vec![3., 4.]]).serialize(),
            MULTI_DIMENSION_2_DIM_BYTES
        )
    }

    #[test]
    fn can_deserialise_multi_dimension_3_dimension() {
        let cube = PgCube::deserialize(MULTI_DIMENSION_3_DIM_BYTES).unwrap();
        assert_eq!(
            cube,
            PgCube::MultiDimension(vec![vec![2., 3., 4.], vec![5., 6., 7.]])
        )
    }
    #[test]
    fn can_serialise_multi_dimension_3_dimension() {
        assert_eq!(
            PgCube::MultiDimension(vec![vec![2., 3., 4.], vec![5., 6., 7.]]).serialize(),
            MULTI_DIMENSION_3_DIM_BYTES
        )
    }
}
