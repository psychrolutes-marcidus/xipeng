use geo::{coord, point, Distance, InterpolatePoint};
use geo_traits::{CoordTrait, GeometryTrait, LineTrait};

pub fn line_to_aabb_triangles<Line:LineTrait + std::fmt::Debug>(line: &Line, a: f64, b: f64, c: f64, d: f64) -> Vec<geo_types::Triangle> 
where 
    Line: GeometryTrait<T = f64>
{
    let dx = line.end().x() - line.start().x();
    let dy = line.end().y() - line.start().y();
    //let dnorm = f64::sqrt(dx*dx+dy*dy);

    //let vec_orth_c = vec![-dy, dx]; // orthogonal vector of the line, representative of c width
    //let vec_orth_d = vec![dy, -dx]; // orthogonal vector of the line, representative of d width

    let start_point_c = geo::algorithm::line_measures::metric_spaces::Geodesic.point_at_distance_between(
        point!(line.start().x_y()),
        point!(x: line.start().x()+(-dy), y: line.start().y()+(dx)),
        c);

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

    let p1 = point!(x: 8.15, y: 56.1);
    let x_1 = line.start().x();
    let y_1 = line.start().y();
    let x_2 = line.end().x();
    let y_2 = line.end().y();
    let top = f64::abs((y_2-y_1)*p1.x()-(x_2-x_1)*p1.y()+(x_2*y_1)-(y_2*x_1));
    let bottom = f64::sqrt(f64::powi(y_2-y_1, 2)+f64::powi(x_2-x_1, 2));

    let top2 = f64::abs((dy*p1.x())-(dx*p1.y())+(x_2*y_1)-(y_2*x_1));
    dbg!(top);
    dbg!(top2);
    let dist_from_point_to_line = (top/bottom);
    dbg!(dist_from_point_to_line);

    vec![
        geo_types::geometry::Triangle::new(start_point_c.0, start_point_d.0, end_point_c.0),
        geo_types::geometry::Triangle::new(start_point_d.0, end_point_c.0, end_point_d.0)
    ]

}