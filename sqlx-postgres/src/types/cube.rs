use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use sqlx_core::Error;
use std::str::FromStr;

const BYTE_WIDTH: usize = 8;
const CUBE_TYPE_ZERO_VOLUME: usize = 128;
const CUBE_TYPE_DEFAULT: usize = 0;
const CUBE_DIMENSION_ONE: usize = 1;
const DIMENSIONALITY_POSITION: usize = 3;
const START_INDEX: usize = 4;

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

impl PgHasArrayType for PgCube {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_cube")
    }
}

impl<'r> Decode<'r, Postgres> for PgCube {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match value.format() {
            PgValueFormat::Text => Ok(PgCube::from_str(value.as_str()?)?),
            PgValueFormat::Binary => Ok(pg_cube_from_bytes(value.as_bytes()?)?),
        }
    }
}

impl<'q> Encode<'q, Postgres> for PgCube {
    fn produces(&self) -> Option<PgTypeInfo> {
        Some(PgTypeInfo::with_name("cube"))
    }

    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        self.serialize(buf)?;
        Ok(IsNull::No)
    }
}

impl FromStr for PgCube {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let content = s
            .trim_start_matches('(')
            .trim_start_matches('[')
            .trim_end_matches(')')
            .trim_end_matches(']')
            .replace(' ', "");

        if !content.contains('(') && !content.contains(',') {
            return parse_point(&content);
        }

        if !content.contains("),(") {
            return parse_zero_volume(&content);
        }

        let point_vecs = content.split("),(").collect::<Vec<&str>>();
        if point_vecs.len() == 2 && !point_vecs.iter().any(|pv| pv.contains(',')) {
            return parse_one_dimensional_interval(point_vecs);
        }

        parse_multidimensional_interval(point_vecs)
    }
}

fn pg_cube_from_bytes(bytes: &[u8]) -> Result<PgCube, Error> {
    let cube_type = bytes
        .first()
        .map(|&byte| byte as usize)
        .ok_or(Error::Decode(
            format!("Could not decode cube bytes: {:?}", bytes).into(),
        ))?;

    let dimensionality = bytes
        .get(DIMENSIONALITY_POSITION)
        .map(|&byte| byte as usize)
        .ok_or(Error::Decode(
            format!("Could not decode cube bytes: {:?}", bytes).into(),
        ))?;

    match (cube_type, dimensionality) {
        (CUBE_TYPE_ZERO_VOLUME, CUBE_DIMENSION_ONE) => {
            let point = get_f64_from_bytes(bytes, 4)?;
            Ok(PgCube::Point(point))
        }
        (CUBE_TYPE_ZERO_VOLUME, _) => {
            Ok(PgCube::ZeroVolume(deserialize_vector(bytes, START_INDEX)?))
        }
        (CUBE_TYPE_DEFAULT, CUBE_DIMENSION_ONE) => {
            let x_start = 4;
            let y_start = x_start + BYTE_WIDTH;
            let x = get_f64_from_bytes(bytes, x_start)?;
            let y = get_f64_from_bytes(bytes, y_start)?;
            Ok(PgCube::OneDimensionInterval(x, y))
        }
        (CUBE_TYPE_DEFAULT, dim) => Ok(PgCube::MultiDimension(deserialize_matrix(
            bytes,
            START_INDEX,
            dim,
        )?)),
        (flag, dimension) => Err(Error::Decode(
            format!(
                "Could not deserialise cube with flag {} and dimension {}: {:?}",
                flag, dimension, bytes
            )
            .into(),
        )),
    }
}

impl PgCube {
    fn serialize(&self, buff: &mut PgArgumentBuffer) -> Result<(), Error> {
        match self {
            PgCube::Point(value) => {
                buff.extend(&[CUBE_TYPE_ZERO_VOLUME as u8, 0, 0, CUBE_DIMENSION_ONE as u8]);
                buff.extend_from_slice(&value.to_be_bytes());
            }
            PgCube::ZeroVolume(values) => {
                let dimension = values.len() as u8;
                buff.extend_from_slice(&[CUBE_TYPE_ZERO_VOLUME as u8, 0, 0]);
                buff.extend_from_slice(&dimension.to_be_bytes());
                let bytes = values
                    .iter()
                    .flat_map(|v| v.to_be_bytes())
                    .collect::<Vec<u8>>();
                buff.extend_from_slice(&bytes);
            }
            PgCube::OneDimensionInterval(x, y) => {
                buff.extend_from_slice(&[0, 0, 0, CUBE_DIMENSION_ONE as u8]);
                buff.extend_from_slice(&x.to_be_bytes());
                buff.extend_from_slice(&y.to_be_bytes());
            }
            PgCube::MultiDimension(multi_values) => {
                let dimension = multi_values
                    .first()
                    .map(|arr| arr.len() as u8)
                    .unwrap_or(1_u8);
                buff.extend_from_slice(&[0, 0, 0]);
                buff.extend_from_slice(&dimension.to_be_bytes());
                let bytes = multi_values
                    .iter()
                    .flatten()
                    .flat_map(|v| v.to_be_bytes())
                    .collect::<Vec<u8>>();
                buff.extend_from_slice(&bytes);
            }
        };
        Ok(())
    }

    #[cfg(test)]
    fn serialize_to_vec(&self) -> Vec<u8> {
        let mut buff = PgArgumentBuffer::default();
        self.serialize(&mut buff).unwrap();
        buff.to_vec()
    }
}

fn get_f64_from_bytes(bytes: &[u8], start: usize) -> Result<f64, Error> {
    bytes
        .get(start..start + BYTE_WIDTH)
        .ok_or(Error::Decode(
            format!("Could not decode cube bytes: {:?}", bytes).into(),
        ))?
        .try_into()
        .map(f64::from_be_bytes)
        .map_err(|err| Error::Decode(format!("Invalid bytes slice: {:?}", err).into()))
}

fn deserialize_vector(bytes: &[u8], start_index: usize) -> Result<Vec<f64>, Error> {
    let steps = (bytes.len() - start_index) / BYTE_WIDTH;
    (0..steps)
        .map(|i| get_f64_from_bytes(bytes, start_index + i * BYTE_WIDTH))
        .collect()
}

fn deserialize_matrix(
    bytes: &[u8],
    start_index: usize,
    dim: usize,
) -> Result<Vec<Vec<f64>>, Error> {
    let step = BYTE_WIDTH * dim;
    let steps = (bytes.len() - start_index) / step;

    (0..steps)
        .map(|step_idx| {
            (0..dim)
                .map(|dim_idx| {
                    get_f64_from_bytes(bytes, start_index + step_idx * step + dim_idx * BYTE_WIDTH)
                })
                .collect()
        })
        .collect()
}

fn parse_float_from_str(s: &str, error_msg: &str) -> Result<f64, Error> {
    s.parse().map_err(|_| Error::Decode(error_msg.into()))
}

fn parse_point(str: &str) -> Result<PgCube, Error> {
    Ok(PgCube::Point(parse_float_from_str(
        str,
        "Failed to parse point",
    )?))
}

fn parse_zero_volume(content: &str) -> Result<PgCube, Error> {
    content
        .split(',')
        .map(|p| parse_float_from_str(p, "Failed to parse into zero-volume cube"))
        .collect::<Result<Vec<_>, _>>()
        .map(PgCube::ZeroVolume)
}

fn parse_one_dimensional_interval(point_vecs: Vec<&str>) -> Result<PgCube, Error> {
    let x = parse_float_from_str(
        &remove_parentheses(point_vecs.first().ok_or(Error::Decode(
            format!("Could not decode cube interval x: {:?}", point_vecs).into(),
        ))?),
        "Failed to parse X in one-dimensional interval",
    )?;
    let y = parse_float_from_str(
        &remove_parentheses(point_vecs.get(1).ok_or(Error::Decode(
            format!("Could not decode cube interval y: {:?}", point_vecs).into(),
        ))?),
        "Failed to parse Y in one-dimensional interval",
    )?;
    Ok(PgCube::OneDimensionInterval(x, y))
}

fn parse_multidimensional_interval(point_vecs: Vec<&str>) -> Result<PgCube, Error> {
    point_vecs
        .iter()
        .map(|&point_vec| {
            point_vec
                .split(',')
                .map(|point| {
                    parse_float_from_str(
                        &remove_parentheses(point),
                        "Failed to parse into multi-dimension cube",
                    )
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()
        .map(PgCube::MultiDimension)
}

fn remove_parentheses(s: &str) -> String {
    s.trim_matches(|c| c == '(' || c == ')').to_string()
}

#[cfg(test)]
mod cube_tests {

    use std::str::FromStr;

    use crate::types::{cube::pg_cube_from_bytes, PgCube};

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
    fn can_deserialise_point_type_byes() {
        let cube = pg_cube_from_bytes(POINT_BYTES).unwrap();
        assert_eq!(cube, PgCube::Point(2.))
    }

    #[test]
    fn can_deserialise_point_type_str() {
        let cube_1 = PgCube::from_str("(2)").unwrap();
        assert_eq!(cube_1, PgCube::Point(2.));
        let cube_2 = PgCube::from_str("2").unwrap();
        assert_eq!(cube_2, PgCube::Point(2.));
    }

    #[test]
    fn can_serialise_point_type() {
        assert_eq!(PgCube::Point(2.).serialize_to_vec(), POINT_BYTES,)
    }
    #[test]
    fn can_deserialise_zero_volume_bytes() {
        let cube = pg_cube_from_bytes(ZERO_VOLUME_BYTES).unwrap();
        assert_eq!(cube, PgCube::ZeroVolume(vec![2., 3.]));
    }

    #[test]
    fn can_deserialise_zero_volume_string() {
        let cube_1 = PgCube::from_str("(2,3,4)").unwrap();
        assert_eq!(cube_1, PgCube::ZeroVolume(vec![2., 3., 4.]));
        let cube_2 = PgCube::from_str("2,3,4").unwrap();
        assert_eq!(cube_2, PgCube::ZeroVolume(vec![2., 3., 4.]));
    }

    #[test]
    fn can_serialise_zero_volume() {
        assert_eq!(
            PgCube::ZeroVolume(vec![2., 3.]).serialize_to_vec(),
            ZERO_VOLUME_BYTES
        );
    }

    #[test]
    fn can_deserialise_one_dimension_interval_bytes() {
        let cube = pg_cube_from_bytes(ONE_DIMENSIONAL_INTERVAL_BYTES).unwrap();
        assert_eq!(cube, PgCube::OneDimensionInterval(7., 8.))
    }

    #[test]
    fn can_deserialise_one_dimension_interval_string() {
        let cube_1 = PgCube::from_str("((7),(8))").unwrap();
        assert_eq!(cube_1, PgCube::OneDimensionInterval(7., 8.));
        let cube_2 = PgCube::from_str("(7),(8)").unwrap();
        assert_eq!(cube_2, PgCube::OneDimensionInterval(7., 8.));
    }

    #[test]
    fn can_serialise_one_dimension_interval() {
        assert_eq!(
            PgCube::OneDimensionInterval(7., 8.).serialize_to_vec(),
            ONE_DIMENSIONAL_INTERVAL_BYTES
        )
    }

    #[test]
    fn can_deserialise_multi_dimension_2_dimension_byte() {
        let cube = pg_cube_from_bytes(MULTI_DIMENSION_2_DIM_BYTES).unwrap();
        assert_eq!(
            cube,
            PgCube::MultiDimension(vec![vec![1., 2.], vec![3., 4.]])
        )
    }

    #[test]
    fn can_deserialise_multi_dimension_2_dimension_string() {
        let cube_1 = PgCube::from_str("((1,2),(3,4))").unwrap();
        assert_eq!(
            cube_1,
            PgCube::MultiDimension(vec![vec![1., 2.], vec![3., 4.]])
        );
        let cube_2 = PgCube::from_str("((1, 2), (3, 4))").unwrap();
        assert_eq!(
            cube_2,
            PgCube::MultiDimension(vec![vec![1., 2.], vec![3., 4.]])
        );
        let cube_3 = PgCube::from_str("(1,2),(3,4)").unwrap();
        assert_eq!(
            cube_3,
            PgCube::MultiDimension(vec![vec![1., 2.], vec![3., 4.]])
        );
        let cube_4 = PgCube::from_str("(1, 2), (3, 4)").unwrap();
        assert_eq!(
            cube_4,
            PgCube::MultiDimension(vec![vec![1., 2.], vec![3., 4.]])
        )
    }

    #[test]
    fn can_serialise_multi_dimension_2_dimension() {
        assert_eq!(
            PgCube::MultiDimension(vec![vec![1., 2.], vec![3., 4.]]).serialize_to_vec(),
            MULTI_DIMENSION_2_DIM_BYTES
        )
    }

    #[test]
    fn can_deserialise_multi_dimension_3_dimension_bytes() {
        let cube = pg_cube_from_bytes(MULTI_DIMENSION_3_DIM_BYTES).unwrap();
        assert_eq!(
            cube,
            PgCube::MultiDimension(vec![vec![2., 3., 4.], vec![5., 6., 7.]])
        )
    }

    #[test]
    fn can_deserialise_multi_dimension_3_dimension_string() {
        let cube = PgCube::from_str("((2,3,4),(5,6,7))").unwrap();
        assert_eq!(
            cube,
            PgCube::MultiDimension(vec![vec![2., 3., 4.], vec![5., 6., 7.]])
        );
        let cube_2 = PgCube::from_str("(2,3,4),(5,6,7)").unwrap();
        assert_eq!(
            cube_2,
            PgCube::MultiDimension(vec![vec![2., 3., 4.], vec![5., 6., 7.]])
        );
    }

    #[test]
    fn can_serialise_multi_dimension_3_dimension() {
        assert_eq!(
            PgCube::MultiDimension(vec![vec![2., 3., 4.], vec![5., 6., 7.]]).serialize_to_vec(),
            MULTI_DIMENSION_3_DIM_BYTES
        )
    }
}
