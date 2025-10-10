use geo::coord;
use geo_traits::{CoordTrait, GeometryTrait, LineTrait};

pub fn line_to_aabb_triangles<Line:LineTrait + std::fmt::Debug>(line: &Line, a: f64, b: f64, c: f64, d: f64) -> Vec<geo_types::Triangle> 
where 
    Line: GeometryTrait<T = f64>
{
    let dx = line.end().x() - line.start().x();
    let dy = line.end().y() - line.start().y();
    let dnorm = f64::sqrt(dx*dx+dy*dy);
    let ndx = dx/dnorm;
    let ndy = dy/dnorm;
    let vec_orth_c = vec![-ndy*c, ndx*c]; // vec_orth_c/d project their length orthogonally along the normal vector of the line by the given lengths c,d respectively
    let vec_orth_d = vec![ndy*d, -ndx*d];
    let c_coord = coord![x: line.start().x()+vec_orth_c.first().unwrap(), y: line.start().y()+vec_orth_c.last().unwrap()]; // coordinate of left (port) of line AABB from line start
    let d_coord = coord![x: line.start().x()+vec_orth_d.first().unwrap(), y: line.start().y()+vec_orth_d.last().unwrap()]; // coordinate of right (startboard) of line AABB from line start
    let c_coord_end = coord![x: line.end().x()+vec_orth_c.first().unwrap(), y: line.end().y()+vec_orth_c.last().unwrap()]; // --||-- line end
    let d_coord_end = coord![x: line.end().x()+vec_orth_d.first().unwrap(), y: line.end().y()+vec_orth_d.last().unwrap()]; // --||-- line end

    vec![
        geo_types::geometry::Triangle::new(c_coord, d_coord, c_coord_end),
        geo_types::geometry::Triangle::new(d_coord, c_coord_end, d_coord_end)
    ]
}