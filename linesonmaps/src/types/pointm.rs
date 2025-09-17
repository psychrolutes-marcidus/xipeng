use geo::algorithm::Distance;
use geo::{Euclidean, HaversineMeasure};
use geo_traits::CoordTrait;
use geo_traits::{
    GeometryTrait, PointTrait, UnimplementedGeometryCollection, UnimplementedLine,
    UnimplementedLineString, UnimplementedMultiLineString, UnimplementedMultiPoint,
    UnimplementedMultiPolygon, UnimplementedPolygon, UnimplementedRect, UnimplementedTriangle,
};
// use geo::algorithm::Geodesic;
use crate::types::coordm::CoordM;
use geo::algorithm::GeodesicMeasure;
use geo_types::{Coord, Point};
use geographiclib_rs::Geodesic;

///largely similar to a [`CoordM`], but distinctions are made in libraries, so i am going to as well :)
#[derive(Debug, Clone, Copy, PartialEq,Hash)]
pub struct PointM<const CRS: u64 = 4326> {
    pub coord: CoordM<CRS>,
}

impl<const CRS: u64> PointM<CRS> {}

impl<const CRS: u64> From<(f64, f64, f64)> for PointM<CRS> {
    fn from((first, second, third): (f64, f64, f64)) -> Self {
        PointM {
            coord: CoordM {
                x: first,
                y: second,
                m: third,
            },
        }
    }
}
impl<const CRS: u64> From<CoordM<CRS>> for PointM<CRS> {
    fn from(value: CoordM<CRS>) -> Self {
        PointM { coord: value }
    }
}

// maybe this impl can be combined with its nonborrowing equivalent
impl<const CRS: u64> From<&CoordM<CRS>> for PointM<CRS> {
    fn from(value: &CoordM<CRS>) -> Self {
        PointM {
            coord: value.to_owned(),
        }
    }
}

//wth is this
impl<const CRS: u64> GeometryTrait for PointM<CRS> {
    type T = f64;

    type PointType<'a>
        = PointM<CRS>
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

impl<const CRS: u64> PointTrait for PointM<CRS> {
    type CoordType<'a>
        = CoordM<CRS>
    where
        Self: 'a;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(self.coord)
    }
}
impl<const CRS: u64> From<PointM<CRS>> for Point {
    fn from(value: PointM<CRS>) -> Self {
        Point(Coord {
            x: value.coord.x(),
            y: value.coord.y(),
        })
    }
}

impl<const CRS: u64> Distance<f64, PointM<CRS>, PointM<CRS>> for GeodesicMeasure<fn() -> Geodesic> {
    fn distance(&self, origin: PointM<CRS>, destination: PointM<CRS>) -> f64 {
        debug_assert!(
            super::consts::DEGREE_CRS.contains(&CRS),
            "Given CRS: {0} uses non-degree Uom",
            CRS
        );
        self.distance(Point::from(origin), Point::from(destination))
    }
}

impl<const CRS: u64> Distance<f64, PointM<CRS>, PointM<CRS>> for HaversineMeasure {
    fn distance(&self, origin: PointM<CRS>, destination: PointM<CRS>) -> f64 {
        debug_assert!(
            super::consts::DEGREE_CRS.contains(&CRS),
            "Given CRS: {0} uses non-degree Uom",
            CRS
        );
        self.distance(Point::from(origin), Point::from(destination))
    }
}

impl<const CRS: u64> Distance<f64, PointM<CRS>, PointM<CRS>> for Euclidean {
    fn distance(&self, origin: PointM<CRS>, destination: PointM<CRS>) -> f64 {
        debug_assert!(
            super::consts::METRIC_CRS.contains(&CRS),
            "Given CRS: {0} uses non-meter Uom",
            CRS
        );
        self.distance(Point::from(origin), Point::from(destination))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::algorithm::line_measures::metric_spaces::Geodesic;
    use pretty_assertions::{assert_eq, assert_ne};
    use proj::Proj;

    #[test]
    fn geodesic_distance() {
        let first = PointM::<4326>::from((1.0, 2.0, 0.0));
        let second = PointM::from((1.0, 3.0, 1.0));
        let zero_dist = GeodesicMeasure::wgs84().distance(first, first);
        assert_eq!(zero_dist, 0.0);
        let dist = GeodesicMeasure::wgs84().distance(first, second);
        assert!(dist >= 11_000.);

        // the preferred way of measuring geodesic distance
        let alternative = Geodesic.distance(first, second);
        assert_eq!(dist, alternative);
    }

    #[test]
    #[ignore = "does not work the way i thought"]
    fn proj_is_projing() {
        dbg!(
            Proj::new_known_crs("EPSG:3857", "EPSG:4326", None)
                .unwrap()
                .proj_info()
        );
        assert!(false)
    }
}
