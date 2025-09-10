use geo_traits::{GeometryTrait, GeometryType, LineStringTrait, UnimplementedGeometryCollection, UnimplementedLine, UnimplementedMultiLineString, UnimplementedMultiPoint, UnimplementedMultiPolygon, UnimplementedPolygon, UnimplementedRect, UnimplementedTriangle};

use crate::types::coordm::CoordM;
use crate::types::pointm::PointM;

#[derive(Debug,Clone,PartialEq)]
pub struct LineStringM(Vec<CoordM>);

impl LineStringM{}

impl TryFrom<Vec<CoordM>> for LineStringM{
    type Error = (); //TODO

    fn try_from(value: Vec<CoordM>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl LineStringTrait for LineStringM{
    type CoordType<'a> = CoordM
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

impl GeometryTrait for LineStringM{
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