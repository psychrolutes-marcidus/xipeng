use std::vec;

use geo::{coord, point, CoordFloat, Distance, Geodesic, GeodesicMeasure, InterpolatableLine, InterpolatePoint, Length, Vector2DOps};
use geo_traits::{CoordTrait, GeometryTrait, LineTrait, MultiPointTrait, PointTrait};
use linesonmaps::types::linem::LineM;


pub fn line_to_aabb_triangles<const CRS: u64>(line: &LineM<CRS>, a: f64, b: f64, c: f64, d: f64) -> (geo_types::geometry::Triangle, geo_types::geometry::Triangle, impl Fn(geo_types::geometry::Triangle ,f64, f64, f64) -> Vec<f64>, impl Fn() -> f64
) {
    let dx = line.end().x() - line.start().x();
    let dy = line.end().y() - line.start().y();
    //let dnorm = f64::sqrt(dx*dx+dy*dy);

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


    let p1 = point!(x: 8.00, y: 56.00); // probe point
    let p1_vec = vec![p1.x()-line.start().x(), p1.y()-line.start().y()]; // vector from line start to probe point
    let p1_dot_line = f64::abs(p1_vec[0]*dx+p1_vec[1]*dy);
    let line_length = f64::sqrt(f64::powi(dx, 2)+ f64::powi(dy, 2));
    let proj_p1_vec_length = p1_dot_line / line_length;  // length of p1_vec projected onto the line
    let ratio = proj_p1_vec_length/line_length;
    let proj_p1 = Geodesic.point_at_ratio_between(
        point!(x: line.start().x(), y: line.start().y()),
        point!(x: line.end().x(), y: line.end().y()),
        ratio); // The probe point projected onto the line

    dbg!(proj_p1);


    // calculating probe m
    let delta_m = line.end().m - line.start().m;
    let probe_m = delta_m*ratio;
    dbg!(probe_m);

    // approximating ship occupation time at probe location
    let line_meters = geo::algorithm::line_measures::metric_spaces::Geodesic.distance(line.from, line.to);
    dbg!(line_meters);
    let p1_occupation = delta_m/line_meters*(a+b);
    dbg!(p1_occupation);
    // 20s / 100m = 0.2s/m
    // 0.2s/m * 40m = 8s
    // Explanation: If delta_m = 20s, and line_meters is 100m, the ship is travelling at 0.2s/m
    // This means that to travel 40m (eg length of ship) it takes 8s.

    // compute barycentric coords for probe point
    let triangle1 = geo_types::geometry::Triangle::new(start_point_c.0, start_point_d.0, end_point_c.0).to_array();
    let triangle2 = geo_types::geometry::Triangle::new(start_point_d.0, end_point_c.0, end_point_d.0).to_array();

    dbg!(triangle1);
    let v0 = triangle1[1]-triangle1[0];
    let v1 = triangle1[2]-triangle1[0];
    let v2 = p1.coord().unwrap()-triangle1[0];
    let den = v0.x() * v1.y() - v1.x() * v0.y();
    let v = (v2.x() * v1.y() - v1.x() * v2.y()) / den;
    let w = (v0.x() * v2.y() - v2.x() * v0.y()) / den;
    let u = 1.0 - v - w;
    dbg!(u, v, w); // barycentric coordinates of triangle1 depicting the location of probe point 
    
    // Conversion from barycentric coords to cartesian
    //dbg!(u*triangle1[0].x() + v*triangle1[1].x() + w*triangle1[2].x()); // x
    //dbg!(u*triangle1[0].y() + v*triangle1[1].y() + w*triangle1[2].y()); // y

    let barycentric_to_lat_long =  |triangle:geo_types::geometry::Triangle, ba: f64, bb: f64, bc: f64| -> Vec<f64> {
        vec![
            ba*triangle.0.x() + bb*triangle.1.x() + bc*triangle.2.x(),
            ba*triangle.0.y() + bb*triangle.1.y() + bc*triangle.2.y()
        ]
    };

    let occupation_time = move || -> f64 {(line.end().m - line.start().m)/geo::algorithm::line_measures::metric_spaces::Geodesic.distance(line.from, line.to)*(a+b)}; // delta_m(s)/line_meters(m)*length(m)=occupation_time(s)

     
    (
        geo_types::geometry::Triangle::new(start_point_c.0, start_point_d.0, end_point_c.0),
        geo_types::geometry::Triangle::new(start_point_d.0, end_point_c.0, end_point_d.0),
        barycentric_to_lat_long,
        occupation_time
    )
}