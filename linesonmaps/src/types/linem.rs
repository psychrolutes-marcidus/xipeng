use crate::types::coordm::CoordM;
use crate::types::linestringm::LineStringM;
use crate::types::multilinestringm::MultiLineStringM;
use crate::types::pointm::PointM;
use crate::*;
use geo_traits::{
    CoordTrait, GeometryTrait, GeometryType, LineStringTrait, LineTrait, MultiLineStringTrait, UnimplementedGeometryCollection, UnimplementedLine, UnimplementedMultiLineString, UnimplementedMultiPoint, UnimplementedMultiPolygon, UnimplementedPolygon, UnimplementedRect, UnimplementedTriangle
};


#[derive(Debug, Clone, Copy, PartialEq,Hash)]
pub struct LineM<const CRS: u64= 4326> {pub from: PointM<CRS>, pub to: PointM<CRS>}

impl<const CRS:u64> From<(PointM<CRS>,PointM<CRS>)> for LineM<CRS>{
    fn from(value: (PointM<CRS>,PointM<CRS>)) -> Self {
        LineM { from: value.0, to: value.1 }
    }
}

impl<const CRS:u64> From<(CoordM<CRS>,CoordM<CRS>)> for LineM<CRS> {
    fn from(value: (CoordM<CRS>,CoordM<CRS>)) -> Self {
        LineM { from: value.0.into(), to: value.1.into() }
    }
}

impl<const CRS: u64> LineTrait for LineM<CRS>{
    type CoordType<'a> = CoordM<CRS>
    where
        Self: 'a;

    fn start(&self) -> Self::CoordType<'_> {
        self.from.coord
    }

    fn end(&self) -> Self::CoordType<'_> {
        self.to.coord
    }
}

impl<const CRS: u64>GeometryTrait for LineM<CRS>{
    type T = f64;

    type PointType<'a>
        = PointM<CRS>
    where
        Self: 'a;

    type LineStringType<'a>
        = LineStringM<CRS>
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
        = MultiLineStringM<CRS>
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
        = LineM<CRS>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xym
    }

    fn as_type(
        &self,
    ) -> GeometryType<
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
        GeometryType::Line(self)
    }
}