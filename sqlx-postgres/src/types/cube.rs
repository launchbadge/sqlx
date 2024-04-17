use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::types::Type;
use crate::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use sqlx_core::Error;

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
        match value.format() {
            PgValueFormat::Text => Ok(PgCube::try_from(value.as_str()?)?),
            PgValueFormat::Binary => Ok(PgCube::try_from(value.as_bytes()?)?),
        }
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
        .map(|i| get_f64_from_bytes(&bytes, start_index + i * BYTE_WIDTH))
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
                    get_f64_from_bytes(&bytes, start_index + step_idx * step + dim_idx * BYTE_WIDTH)
                })
                .collect()
        })
        .collect()
}

fn parse_float_from_str(s: &str, error_msg: &str) -> Result<f64, Error> {
    s.trim()
        .parse()
        .map_err(|_| Error::Decode(error_msg.into()))
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
        &remove_parentheses(point_vecs.get(0).ok_or(Error::Decode(
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

impl TryFrom<&str> for PgCube {
    type Error = Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let content = &input.get(1..input.len() - 1).ok_or(Error::Decode(
            format!("Could not decode cube string: {}", input).into(),
        ))?;

        if !content.contains('(') && !content.contains(',') {
            return parse_point(content);
        }

        if !content.contains("),(") {
            return parse_zero_volume(content);
        }

        let point_vecs = content.split("),(").collect::<Vec<&str>>();
        if point_vecs.len() == 2 && !point_vecs.iter().any(|pv| pv.contains(',')) {
            return parse_one_dimensional_interval(point_vecs);
        }

        parse_multidimensional_interval(point_vecs)
    }
}

impl TryFrom<&[u8]> for PgCube {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let cube_type = bytes
            .get(0)
            .map(|&byte| byte as usize)
            .ok_or(Error::Decode(
                format!("Could not decode cube bytes: {:?}", bytes).into(),
            ))?;

        let dimensionality = bytes
            .get(3)
            .map(|&byte| byte as usize)
            .ok_or(Error::Decode(
                format!("Could not decode cube bytes: {:?}", bytes).into(),
            ))?;

        let start_index = 4;

        match (cube_type, dimensionality) {
            (128, 1) => {
                let point = get_f64_from_bytes(&bytes, 4)?;
                Ok(PgCube::Point(point))
            }
            (128, _) => Ok(PgCube::ZeroVolume(deserialize_vector(&bytes, start_index)?)),
            (0, 1) => {
                let x_start = 4;
                let y_start = x_start + BYTE_WIDTH;
                let x = get_f64_from_bytes(&bytes, x_start)?;
                let y = get_f64_from_bytes(&bytes, y_start)?;
                Ok(PgCube::OneDimensionInterval(x, y))
            }
            (0, dim) => Ok(PgCube::MultiDimension(deserialize_matrix(
                &bytes,
                start_index,
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
}

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
    fn can_deserialise_point_type_byes() {
        let cube = PgCube::try_from(POINT_BYTES).unwrap();
        assert_eq!(cube, PgCube::Point(2.))
    }

    #[test]
    fn can_deserialise_point_type_str() {
        let cube = PgCube::try_from("(2)").unwrap();
        assert_eq!(cube, PgCube::Point(2.))
    }

    #[test]
    fn can_serialise_point_type() {
        assert_eq!(PgCube::Point(2.).serialize(), POINT_BYTES,)
    }
    #[test]
    fn can_deserialise_zero_volume_bytes() {
        let cube = PgCube::try_from(ZERO_VOLUME_BYTES).unwrap();
        assert_eq!(cube, PgCube::ZeroVolume(vec![2., 3.]));
    }

    #[test]
    fn can_deserialise_zero_volume_string() {
        let cube = PgCube::try_from("(2,3,4)").unwrap();
        assert_eq!(cube, PgCube::ZeroVolume(vec![2., 3., 4.]));
    }

    #[test]
    fn can_serialise_zero_volume() {
        assert_eq!(
            PgCube::ZeroVolume(vec![2., 3.]).serialize(),
            ZERO_VOLUME_BYTES
        );
    }

    #[test]
    fn can_deserialise_one_dimension_interval_bytes() {
        let cube = PgCube::try_from(ONE_DIMENSIONAL_INTERVAL_BYTES).unwrap();
        assert_eq!(cube, PgCube::OneDimensionInterval(7., 8.))
    }

    #[test]
    fn can_deserialise_one_dimension_interval_string() {
        let cube = PgCube::try_from("((7),(8))").unwrap();
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
    fn can_deserialise_multi_dimension_2_dimension_byte() {
        let cube = PgCube::try_from(MULTI_DIMENSION_2_DIM_BYTES).unwrap();
        assert_eq!(
            cube,
            PgCube::MultiDimension(vec![vec![1., 2.], vec![3., 4.]])
        )
    }

    #[test]
    fn can_deserialise_multi_dimension_2_dimension_string() {
        let cube = PgCube::try_from("((1,2),(3,4))").unwrap();
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
    fn can_deserialise_multi_dimension_3_dimension_bytes() {
        let cube = PgCube::try_from(MULTI_DIMENSION_3_DIM_BYTES).unwrap();
        assert_eq!(
            cube,
            PgCube::MultiDimension(vec![vec![2., 3., 4.], vec![5., 6., 7.]])
        )
    }

    #[test]
    fn can_deserialise_multi_dimension_3_dimension_string() {
        let cube = PgCube::try_from("((2,3,4),(5,6,7))").unwrap();
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
