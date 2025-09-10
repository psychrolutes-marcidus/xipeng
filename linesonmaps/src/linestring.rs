use geo_traits::{
    CoordTrait, GeometryTrait, GeometryType, LineStringTrait, PointTrait, UnimplementedGeometryCollection, UnimplementedLine, UnimplementedLineString, UnimplementedMultiLineString, UnimplementedMultiPoint, UnimplementedMultiPolygon, UnimplementedPolygon, UnimplementedRect, UnimplementedTriangle
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoordM {
    pub x: f64,
    pub y: f64,
    pub m: f64,
}

impl CoordM {}

impl CoordTrait for CoordM {
    type T = f64;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xym
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x,
            1 => self.y,
            2 => (self.m),
            e => panic!("tried to access dimension {e} in 3-dimensional coordinate"),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointM {
    pub coord: CoordM,
}

impl PointM {}

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

#[derive(Debug,Clone,PartialEq)]
pub struct LineStringM(Vec<CoordM>);

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