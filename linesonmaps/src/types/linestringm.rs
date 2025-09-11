use geo_traits::{
    CoordTrait, GeometryTrait, GeometryType, LineStringTrait, UnimplementedGeometryCollection,
    UnimplementedLine, UnimplementedMultiLineString, UnimplementedMultiPoint,
    UnimplementedMultiPolygon, UnimplementedPolygon, UnimplementedRect, UnimplementedTriangle,
};

use crate::types::coordm::CoordM;
use crate::types::error::Error;
use crate::types::pointm::PointM;

#[derive(Debug, Clone, PartialEq)]
pub struct LineStringM(pub Vec<CoordM>);

impl LineStringM {
    pub fn new(coords: Vec<CoordM>) -> Option<LineStringM> {
        if coords.iter().map(|f| f.m).is_sorted() && coords.len() != 1 {
            Some(LineStringM(coords))
        } else {
            None
        }
    }
}

impl TryFrom<Vec<CoordM>> for LineStringM {
    type Error = super::error::Error;

    fn try_from(value: Vec<CoordM>) -> Result<Self, Self::Error> {
        match value.len() {
            1 => Err(super::error::Error::NumPoints), //TODO verify that points are temporally ordered
            _ => Ok(LineStringM(value)),
        }
    }
}

impl TryFrom<wkb::reader::Wkb<'_>> for LineStringM {
    type Error = super::error::Error;

    fn try_from(value: wkb::reader::Wkb<'_>) -> Result<Self, Self::Error> {
        match value.as_type() {
            geo_traits::GeometryType::LineString(ls) => {
                let coords = ls
                    .coords()
                    .map(|c| {
                        Some(CoordM {
                            x: c.x(),
                            y: c.y(),
                            m: c.nth(2)?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()
                    .ok_or(Error::Dimension)?;
                Ok(LineStringM(coords))
            }
            _ => Err(super::error::Error::IncompatibleType),
        }
    }
}

impl LineStringTrait for LineStringM {
    type CoordType<'a>
        = CoordM
    where
        Self: 'a;

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        // ¬(i also like to live dangerously)
        match i <= self.0.len() {
            true => self.0[i],
            false => panic!("u sux"), //TODO: better error message
        }
    }
}
impl GeometryTrait for LineStringM {
    type T = f64;

    type PointType<'a>
        = PointM
    where
        Self: 'a;

    type LineStringType<'a>
        = LineStringM
    where
        Self: 'a;

    type PolygonType<'a>
        = UnimplementedPolygon<Self::T>
    where
        Self: 'a;

    type MultiPointType<'a>
        = UnimplementedMultiPoint<Self::T>
    where
        Self: 'a;

    type MultiLineStringType<'a>
        = UnimplementedMultiLineString<Self::T>
    where
        Self: 'a;

    type MultiPolygonType<'a>
        = UnimplementedMultiPolygon<Self::T>
    where
        Self: 'a;

    type GeometryCollectionType<'a>
        = UnimplementedGeometryCollection<Self::T>
    where
        Self: 'a;

    type RectType<'a>
        = UnimplementedRect<Self::T>
    where
        Self: 'a;

    type TriangleType<'a>
        = UnimplementedTriangle<Self::T>
    where
        Self: 'a;

    type LineType<'a>
        = UnimplementedLine<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xym
    }

    fn as_type(
        &self,
    ) -> geo_traits::GeometryType<
        '_,
        Self::PointType<'_>,
        Self::LineStringType<'_>,
        Self::PolygonType<'_>,
        Self::MultiPointType<'_>,
        Self::MultiLineStringType<'_>,
        Self::MultiPolygonType<'_>,
        Self::GeometryCollectionType<'_>,
        Self::RectType<'_>,
        Self::TriangleType<'_>,
        Self::LineType<'_>,
    > {
        GeometryType::LineString(self)
    }
}

#[cfg(test)]
mod tests {
    use geo_traits::CoordTrait;
    use geo_traits::{GeometryTrait, LineStringTrait};
    use hex::encode;

    use wkb::reader::GeometryType;
    use wkb::reader::read_wkb;
    use wkb::writer::WriteOptions;
    use wkb::writer::write_line_string;

    use crate::types::coordm::CoordM;
    use crate::types::linestringm::LineStringM;

    #[test]
    fn writer() {
        let mut output: Vec<u8> = Vec::new();

        let coords: Vec<CoordM> = [(1.0, 2.0, 0.0), (2.0, 3.0, 1.0), (3.0, 4.0, 2.0)]
            .map(|f| f.into())
            .to_vec();
        let ls = LineStringM::try_from(coords.clone()).unwrap();
        let _ = write_line_string(
            &mut output,
            &ls,
            &WriteOptions {
                endianness: wkb::Endianness::LittleEndian,
            },
        );

        let hexstring = encode(&output); // should be parsable by wkb reader tools online
        dbg!(hexstring); // https://wkbrew.tszheichoi.com/
        let input = read_wkb(&output).unwrap();
        assert_eq!(input.geometry_type(), GeometryType::LineString);
        let ls = match input.as_type() {
            geo_traits::GeometryType::LineString(ls) => ls,
            _ => unreachable!(),
        };
        assert_eq!(ls.num_coords(), 3);
        let c = ls
            .coords()
            .map(|f| CoordM {
                x: f.x(),
                y: f.y(),
                m: f.nth_or_panic(2),
            })
            .collect::<Vec<_>>();
        assert_eq!(&coords, &c);
    }

    #[test]
    fn to_linestringM() {
        let hexstring = "01d207000003000000000000000000f03f0000000000000040000000000000000000000000000000400000000000000840000000000000f03f000000000000084000000000000010400000000000000040";

        let bytea = hex::decode(hexstring).unwrap();

        let wkb = read_wkb(&bytea).unwrap();
        let lsm = LineStringM::try_from(wkb);
        assert!(lsm.is_ok());
        // dbg!(lsm.unwrap());
    }
}
