use std::{ops::Div};

use chrono::{DateTime, Duration, Utc};
use geo::{coord, point, Coord, Distance, InterpolatePoint, Point, Vector2DOps};
use geo_traits::{CoordTrait, LineTrait, PointTrait};
use linesonmaps::types::{linem::LineM, pointm::PointM};
use geo_types::geometry::Triangle;


pub struct LineTriangle<const CRS: u64> {
    pub triangle: Triangle,
    pub line: LineM<CRS>,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64
}

impl <const CRS: u64> LineTriangle<CRS> {
    pub fn point_occupation(&self, ba: f64, bb: f64, bc: f64) -> (DateTime<Utc>, DateTime<Utc>) {

        let probe_vec = probe_vector(&self.line, self.triangle, ba, bb, bc);
        dbg!(probe_vec);


        let ratio = probe_ratio(probe_vec,
            self.line.end().x() - self.line.start().x(),
            self.line.end().y() - self.line.start().y(),
            );
        dbg!(ratio);

        let probe_m = probe_timestamp(
            self.line.start().m,
            self.line.end().m - self.line.start().m,
            ratio);

        let line_meters = meters_between_points(self.line.from, self.line.to);
        
        probe_occupation(probe_m, self.line.end().m - self.line.start().m, line_meters, self.a, self.b)
    }
}

pub fn barycentric_to_cartesian(triangle:Triangle, ba: f64, bb: f64, bc: f64) -> geo::Coord<f64> {
    coord![
        x: ba*triangle.0.x() + bb*triangle.1.x() + bc*triangle.2.x(),
        y: ba*triangle.0.y() + bb*triangle.1.y() + bc*triangle.2.y()
    ]
}


pub fn line_to_triangle_pair<const CRS: u64>(line: &LineM<CRS>, a: f64, b: f64, c: f64, d: f64) -> (LineTriangle<CRS>, LineTriangle<CRS>) {
    let dx = line.end().x() - line.start().x();
    let dy = line.end().y() - line.start().y();

    //let vec_orth_c = vec![-dy, dx]; // orthogonal vector of the line, representative of c width
    //let vec_orth_d = vec![dy, -dx]; // orthogonal vector of the line, representative of d width

    let start_point_c = geo::algorithm::line_measures::metric_spaces::Geodesic.point_at_distance_between(
        point!(line.start().x_y()),
        point!(x: line.start().x()+(-dy), y: line.start().y()+(dx)),
        c); // Point 
    
    let start_point_d = geo::algorithm::line_measures::metric_spaces::Geodesic.point_at_distance_between(
        point!(line.start().x_y()),
        point!(x: line.start().x()+(dy), y: line.start().y()+(-dx)),
        d);
    
    let end_point_c = geo::algorithm::line_measures::metric_spaces::Geodesic.point_at_distance_between(
        point!(line.end().x_y()),
        point!(x: line.end().x()+(-dy), y: line.end().y()+(dx)),
        c);

    let end_point_d = geo::algorithm::line_measures::metric_spaces::Geodesic.point_at_distance_between(
        point!(line.end().x_y()),
        point!(x: line.end().x()+(dy), y: line.end().y()+(-dx)),
        d);

    (
        LineTriangle{triangle: Triangle::new(start_point_c.0, start_point_d.0, end_point_c.0), line: line.clone(), a: a, b: b, c: c, d: d},
        LineTriangle{triangle: Triangle::new(start_point_d.0, end_point_c.0, end_point_d.0), line: line.clone(), a: a, b: b, c: c, d: d}
    )
}

// Not used for anything :)
pub fn probe_to_barycentric_coordinates(triangle: Triangle, probe: Point) -> (f64, f64, f64) {
    let v0 = triangle.1-triangle.0;
    let v1 = triangle.2-triangle.0;
    let v2 = probe.coord().unwrap()-triangle.0;
    let den = v0.x() * v1.y() - v1.x() * v0.y();
    let v = (v2.x() * v1.y() - v1.x() * v2.y()) / den;
    let w = (v0.x() * v2.y() - v2.x() * v0.y()) / den;
    let u = 1.0 - v - w;
    (u, v, w) // barycentric coordinates of triangle1 depicting the location of probe point
}

pub fn probe_timestamp(start_m: f64, delta_m: f64, ratio: f64) -> DateTime<Utc>{
    DateTime::<Utc>::from_timestamp_secs((delta_m*ratio) as i64 + start_m as i64).unwrap()
}

pub fn probe_occupation(probe_m: DateTime<Utc>, delta_m: f64, line_meters: f64, a: f64, b: f64) -> (DateTime<Utc>, DateTime<Utc>) {
    (
        probe_m - Duration::seconds((delta_m/line_meters*a) as i64), // formula: timestamp - 'how much earlier the ship arrived due to its length infront of sensor'
        probe_m + Duration::seconds((delta_m/line_meters*b) as i64) // formula: timestamp + 'how much longer did the ship stay due to its length behind the sensor'
    )
}

pub fn vector_length(x: f64, y: f64) -> f64 {
    f64::sqrt(f64::powi(x, 2)+ f64::powi(y, 2))
}

pub fn vector_length2(x: f64, y: f64) -> f64 {
    f64::powi(x, 2)+ f64::powi(y, 2)
}

pub fn meters_between_points<const CRS: u64>(origin: PointM<CRS>, destination: PointM<CRS>) -> f64 {
    geo::algorithm::line_measures::metric_spaces::Geodesic.distance(origin, destination)
}

pub fn probe_vector<const CRS: u64>(line: &LineM<CRS>, triangle: Triangle, ba: f64, bb: f64, bc:f64) -> Coord<f64> {
    let coord = barycentric_to_cartesian(triangle, ba, bb, bc);
    dbg!(coord);
    coord! {x: coord.x-line.start().x, y: coord.y-line.start().y}
}

// ratio of how far along the line the probe point is
pub fn probe_ratio(coord: Coord, dx: f64, dy: f64) -> f64 {
    coord.dot_product(coord! {x: dx, y: dy})
        .div(vector_length(dx, dy)) // length of the projected vector, formula: (|a_vec*b_vec|) / |a_vec| = |b_a_vec|
        .div(vector_length(dx, dy)) // projection_length/length = ratio, small optimzation: (x/y)/y == x/(y^2)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use geo::{coord, Distance, Line};
    use linesonmaps::types::{coordm::CoordM, linem::LineM};
    use crate::modeling::line_to_triangle_pair;

    use super::*;

    #[test]
    fn half_way_test() {
        let start_m = DateTime::parse_from_str("2024-01-01 00:00:00 +0000", "%Y-%m-%d %H:%M:%S%.3f %z").unwrap().timestamp() as f64;
        let end_m = DateTime::parse_from_str("2024-01-01 00:02:00 +0000", "%Y-%m-%d %H:%M:%S%.3f %z").unwrap().timestamp() as f64;

        let coords: Vec<CoordM<4326>> = [(8.0, 56.0, start_m), (8.005, 56.0, end_m)]            
            .map(|f| f.into())
            .to_vec();
        let line = LineM::<4326>::from((coords[0], coords[1]));
        line.from.coord.m;

        let a = line_to_triangle_pair(&line, 1.0,1.0,10.0,10.0);
        dbg!(start_m, end_m);
        dbg!(a.0.point_occupation(1./2., 0., 1./2.));
        dbg!((a.0.triangle, a.1.triangle));
        dbg!(meters_between_points(line.from, line.to));
        dbg!(a.0.point_occupation(1./2., 0., 1./2.).0.timestamp() as f64 - start_m);
        assert_eq!(a.0.point_occupation(1./2., 0., 1./2.).0.timestamp() as f64 - start_m, (end_m-start_m)/2.0)
    }
}
