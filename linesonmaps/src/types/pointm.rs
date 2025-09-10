use geo_traits::{GeometryTrait, PointTrait, UnimplementedGeometryCollection, UnimplementedLine, UnimplementedLineString, UnimplementedMultiLineString, UnimplementedMultiPoint, UnimplementedMultiPolygon, UnimplementedPolygon, UnimplementedRect, UnimplementedTriangle};

use crate::types::coordm::CoordM;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointM {
    pub coord: CoordM,
}

impl PointM {}

impl From<(f64,f64,f64)> for PointM {
    fn from((first,second,third): (f64,f64,f64)) -> Self {
        PointM {coord: CoordM { x: first, y: second, m: third }}
    }
}
impl From<CoordM> for PointM{
    fn from(value: CoordM) -> Self {
        PointM { coord: value }
    }
}

//wth is this
impl GeometryTrait for PointM {
    type T = f64;

    type PointType<'a>
        = PointM
    where
        Self: 'a;

    type LineStringType<'a>
        = UnimplementedLineString<Self::T>
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
        geo_traits::GeometryType::Point(self)
        // geo_traits::GeometryType::
    }
}

impl PointTrait for PointM {
    type CoordType<'a>
        = CoordM
    where
        Self: 'a;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(self.coord)
    }
}