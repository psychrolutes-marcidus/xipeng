use geo::{coord, point, CoordFloat, Distance, Geodesic, GeodesicMeasure, InterpolatableLine, InterpolatePoint, Length};
use geo_traits::{CoordTrait, GeometryTrait, LineTrait, MultiPointTrait};
use linesonmaps::types::linem::LineM;


pub fn line_to_aabb_triangles<const CRS: u64>(line: &LineM<CRS>, a: f64, b: f64, c: f64, d: f64) -> Vec<geo_types::Triangle> {
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


    let p1 = point!(x: 8.05, y: 56.02); // probe point
    let p1_vec = vec![p1.x()-line.start().x(), p1.y()-line.start().y()]; // vector from line start to probe point
    let p1_dot_line = f64::abs(p1_vec[0]*dx+p1_vec[1]*dy);
    let line_length = f64::sqrt(f64::powi(dx, 2)+ f64::powi(dy, 2));
    let proj_p1_vec_length = p1_dot_line / line_length;  // length of p1_vec projected onto the line
    let ratio = proj_p1_vec_length/line_length;
    let proj_p1 = Geodesic.point_at_ratio_between(
        point!(x: line.start().x(), y: line.start().y()),
        point!(x: line.end().x(), y: line.end().y()),
        ratio); // The probe point projected onto the line
    //let dt = line.start().nth(2).unwrap();
    dbg!(proj_p1);
    //dbg!(line.point_at_ratio_from_start(&geo::algorithm::line_measures::Geodesic, proj_p1_vec_length/line_length));
    dbg!(proj_p1_vec_length/line_length);
    dbg!(proj_p1_vec_length);
    dbg!(p1_dot_line);
    dbg!(p1_vec);
    dbg!(line_length);

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
    vec![
        geo_types::geometry::Triangle::new(start_point_c.0, start_point_d.0, end_point_c.0),
        geo_types::geometry::Triangle::new(start_point_d.0, end_point_c.0, end_point_d.0)
    ]

}