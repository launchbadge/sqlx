#[derive(Debug, Clone, PartialEq)]
pub enum Cube {
    Point(f64),
    ZeroVolume(Vec<f64>),
    OneDimensionInterval(f64, f64),
    MultiDimension(Vec<Vec<f64>>),
}

impl sqlx::Type<Postgres> for Cube {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("cube")
    }
}

impl<'r> Decode<'r, Postgres> for Cube {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = value.as_bytes()?;
        Ok(Cube::deserialize(bytes).unwrap())
    }
}
impl Cube {
    fn deserialize(bytes: &[u8]) -> Result<Self, sqlx::Error> {
        let cube_type = bytes[0] as usize;
        let dimensionality = bytes[3] as usize;
        match (cube_type, dimensionality) {
            (128, 1) => {
                let point = f64::from_be_bytes(<[u8; 8]>::try_from(&bytes[4..12]).unwrap());
                Ok(Cube::Point(point))
            }
            (128, _) => {
                let steps = (bytes.len() - 4).div(8);
                let start_index = 4;
                let mut inner_vec = vec![];
                for step_idx in 0..steps {
                    let ll_start = start_index + step_idx * 8;
                    let ll_coord = f64::from_be_bytes(
                        <[u8; 8]>::try_from(&bytes[ll_start..ll_start + 8]).unwrap(),
                    );
                    inner_vec.push(ll_coord);
                }
                Ok(Cube::ZeroVolume(inner_vec))
            }
            (0, 1) => {
                let ll_start = 4; // 16 bytes per dimension (2 coordinates)
                let ur_start = ll_start + 8; // Upper right follows lower left
                let ll_coord = f64::from_be_bytes(
                    <[u8; 8]>::try_from(&bytes[ll_start..ll_start + 8]).unwrap(),
                );
                let ur_coord = f64::from_be_bytes(
                    <[u8; 8]>::try_from(&bytes[ur_start..ur_start + 8]).unwrap(),
                );
                Ok(Cube::OneDimensionInterval(ll_coord, ur_coord))
            }
            (0, dim) => {
                let mut outer_vec = vec![];
                let steps = (bytes.len() - 4).div(8).div(dim);
                let start_index = 4;
                for step_idx in 0..steps {
                    let mut inner_vec = vec![];
                    for dim_idx in 0..dim {
                        let ll_start = start_index + (step_idx * dim * 8) + dim_idx * 8;
                        let ll_coord = f64::from_be_bytes(
                            <[u8; 8]>::try_from(&bytes[ll_start..ll_start + 8]).unwrap(),
                        );
                        inner_vec.push(ll_coord);
                    }
                    outer_vec.push(inner_vec);
                }
                Ok(Cube::MultiDimension(outer_vec))
            }
            (flag, dimension) => Err(sqlx::Error::Decode(
                format!(
                    "Could not deserialise cube with flag {} and dimension {}",
                    flag, dimension,
                )
                .into(),
            )),
        }
    }
}

#[cfg(test)]
mod cube_deserialisation {
    #[test]
    fn can_create_point_type() {
        let cube = Cube::deserialize(&[128, 0, 0, 1, 64, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(cube, Cube::Point(2.))
    }
    #[test]
    fn can_create_zero_volume() {
        let cube = Cube::deserialize(&[
            128, 0, 0, 2, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0,
        ])
        .unwrap();
        assert_eq!(cube, Cube::ZeroVolume(vec![2., 3.]));
    }
    #[test]
    fn can_create_one_dimension_interval() {
        let cube = Cube::deserialize(&[
            0, 0, 0, 1, 64, 28, 0, 0, 0, 0, 0, 0, 64, 32, 0, 0, 0, 0, 0, 0,
        ])
        .unwrap();
        assert_eq!(cube, Cube::OneDimensionInterval(7., 8.))
    }
    #[test]
    fn can_create_multi_dimension_2_dimension() {
        let cube = Cube::deserialize(&[
            0, 0, 0, 2, 63, 240, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0,
            0, 64, 16, 0, 0, 0, 0, 0, 0,
        ])
        .unwrap();
        assert_eq!(cube, Cube::MultiDimension(vec![vec![1., 2.], vec![3., 4.]]))
    }

    #[test]
    fn can_create_multi_dimension_3_dimension() {
        let cube = Cube::deserialize(&[
            0, 0, 0, 3, 64, 0, 0, 0, 0, 0, 0, 0, 64, 8, 0, 0, 0, 0, 0, 0, 64, 16, 0, 0, 0, 0, 0, 0,
            64, 20, 0, 0, 0, 0, 0, 0, 64, 24, 0, 0, 0, 0, 0, 0, 64, 28, 0, 0, 0, 0, 0, 0,
        ])
        .unwrap();
        assert_eq!(
            cube,
            Cube::MultiDimension(vec![vec![2., 3., 4.], vec![5., 6., 7.]])
        )
    }
}
