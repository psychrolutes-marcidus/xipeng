pub mod modeling;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use geo::{coord, Distance, Line};
    use crate::modeling::line_to_aabb_triangles;

    use super::*;

    #[test]
    fn dumb_test() {
        let line = Line::new(coord! { x: 0., y: 0. }, coord! { x: 5., y: 0. });
        /*let coords: Vec<CoordM<4326>> = [(1.0, 2.0, 0.0), (5.0, 3.0, 1.0), (3.0, 4.0, 2.0)]
            .map(|f| f.into())
            .to_vec();
        let first_line = LineM::from((coords[0],coords[1]));
        */

        let a = line_to_aabb_triangles(&line, 1.0,1.0,5.0,1.0);
        dbg!(a);
    }
}
