use crate::types::coordm::CoordM;
use crate::types::error::Error;
use crate::types::linestringm::LineStringM;
use crate::types::pointm::PointM;
use crate::*;
use geo_traits::{
    CoordTrait, GeometryTrait, GeometryType, LineStringTrait, MultiLineStringTrait,
    UnimplementedGeometryCollection, UnimplementedLine, UnimplementedMultiLineString,
    UnimplementedMultiPoint, UnimplementedMultiPolygon, UnimplementedPolygon, UnimplementedRect,
    UnimplementedTriangle,
};

#[derive(Debug, Clone, PartialEq,Hash)]
pub struct MultiLineStringM<const CRS: u64 = 4326>(pub Vec<LineStringM<CRS>>);

impl<const CRS:u64> From<Vec<LineStringM<CRS>>> for MultiLineStringM<CRS> {
    fn from(value: Vec<LineStringM<CRS>>) -> Self {
        MultiLineStringM(value)
    }
}

impl<const CRS: u64> TryFrom<wkb::reader::Wkb<'_>> for MultiLineStringM<CRS> {
    type Error = super::error::Error;

    fn try_from(value: wkb::reader::Wkb<'_>) -> Result<Self, Self::Error> {
        match value.as_type() {
            geo_traits::GeometryType::MultiLineString(mls) => {
                let lss = mls
                    .line_strings()
                    .map(|ls| {
                        ls.coords()
                            .map(|c| {
                                Some(CoordM::<CRS> {
                                    x: c.x(),
                                    y: c.y(),
                                    m: c.nth(2)?,
                                })
                            })
                            .collect::<Option<Vec<_>>>()
                            .map(|vc| LineStringM(vc))
                            .ok_or(Error::Dimension)
                    })
                    .collect::<Result<Vec<_>, super::error::Error>>().map(|vls| MultiLineStringM(vls))?;
                Ok(lss)
            }
            _ => Err(super::error::Error::IncompatibleType),
        }
    }
}

impl<const CRS: u64> MultiLineStringTrait for MultiLineStringM<CRS> {
    type InnerLineStringType<'a>
        = LineStringM<CRS>
    where
        Self: 'a;

    fn num_line_strings(&self) -> usize {
        self.0.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::InnerLineStringType<'_> {
        self.0[i].clone()
    }
}

impl<const CRS: u64> GeometryTrait for MultiLineStringM<CRS> {
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
        geo_traits::GeometryType::MultiLineString(self)
    }
}

//TODO: linestring iterator
