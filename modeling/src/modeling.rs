use std::{ops::Div, vec};

use chrono::{Date, DateTime, Duration, TimeDelta, Utc};
use geo::{coord, point, Coord, CoordFloat, Distance, Geodesic, GeodesicMeasure, InterpolatableLine, InterpolatePoint, Length, Point, Vector2DOps};
use geo_traits::{CoordTrait, GeometryTrait, LineTrait, MultiPointTrait, PointTrait};
use linesonmaps::types::{linem::LineM, pointm::PointM};
use geo_types::geometry::Triangle;


pub struct LineTriangle<F> where F: Fn(Triangle, f64, f64, f64) -> (DateTime<Utc>, DateTime<Utc>){
    pub triangle: Triangle,
    pub point_occupation: F,
}

pub fn barycentric_to_cartesian(triangle:Triangle, ba: f64, bb: f64, bc: f64) -> geo::Coord<f64> {
    coord![
        x: ba*triangle.0.x() + bb*triangle.1.x() + bc*triangle.2.x(),
        y: ba*triangle.0.y() + bb*triangle.1.y() + bc*triangle.2.y()
    ]
}

pub fn line_to_aabb_triangles<const CRS: u64>(line: &LineM<CRS>, a: f64, b: f64, c: f64, d: f64) -> (Triangle, Triangle, impl Fn() -> f64, impl Fn(Triangle, f64, f64, f64) -> (DateTime<Utc>, DateTime<Utc>)
) {
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


    let occupation_time = move || -> f64 {(line.end().m - line.start().m)/geo::algorithm::line_measures::metric_spaces::Geodesic.distance(line.from, line.to)*(a+b)}; // delta_m(s)/line_meters(m)*length(m)=occupation_time(s)

    let occupation_start_end = move |triangle: Triangle, ba: f64, bb: f64, bc: f64| -> (DateTime<Utc>, DateTime<Utc>) {
        let ratio = probe_ratio(dx, dy, triangle, ba, bb, bc);

        let delta_m = line.end().m - line.start().m;

        let probe_m = probe_timestamp(line.start().m, delta_m, ratio);
        dbg!(probe_m);

        let line_meters = meters_between_points(line.from, line.to);
        
        dbg!(probe_occupation(probe_m, delta_m, line_meters, a, b));
        probe_occupation(probe_m, delta_m, line_meters, a, b)
    };

    let l = LineTriangle{triangle: Triangle::new(start_point_c.0, start_point_d.0, end_point_c.0),point_occupation: occupation_start_end};
    (
        Triangle::new(start_point_c.0, start_point_d.0, end_point_c.0),
        Triangle::new(start_point_d.0, end_point_c.0, end_point_d.0),
        occupation_time,
        occupation_start_end
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

pub fn meters_between_points<const CRS: u64>(origin: PointM<CRS>, destination: PointM<CRS>) -> f64 {
    geo::algorithm::line_measures::metric_spaces::Geodesic.distance(origin, destination)
}

// ratio of how far along the line the probe point is
pub fn probe_ratio(dx: f64, dy: f64, triangle: Triangle, ba: f64, bb: f64, bc:f64) -> f64 {
    barycentric_to_cartesian(triangle, ba, bb, bc)
        .dot_product(coord! {x: dx, y: dy})
        .div(vector_length(dx, dy)) // length of the projected vector, formula: (|a_vec*b_vec|) / |a_vec| = |a_b_vec|
        .div(vector_length(dx, dy)) // projection_length/length = ratio
}

#[cfg(test)]
mod tests {
    use geo::{coord, Distance, Line};
    use linesonmaps::types::{coordm::CoordM, linem::LineM};
    use crate::modeling::line_to_aabb_triangles;

    use super::*;

    #[test]
    fn dumb_test() {
        let line = Line::new(coord! { x: 8.0, y: 56.0 }, coord! { x: 8.2, y: 56.2 });
        /*let coords: Vec<CoordM<4326>> = [(1.0, 2.0, 0.0), (5.0, 3.0, 1.0), (3.0, 4.0, 2.0)]
            .map(|f| f.into())
            .to_vec();
        let first_line = LineM::from((coords[0],coords[1]));
        */

        let coords: Vec<CoordM<4326>> = [(8.0, 56.0, 0.0), (8.2, 56.0, 3600.0)]            
            .map(|f| f.into())
            .to_vec();
        let line_m = LineM::<4326>::from((coords[0], coords[1]));
        line_m.from.coord.m;

        let a = line_to_aabb_triangles(&line_m, 300.0,150.0,100.0,100.0);
        dbg!(a.0, a.1);
        dbg!(a.3(a.0, 0., 1., 0.));
        dbg!(a.3(a.1, 0., 1., 0.));
    }
}
