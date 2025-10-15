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
        let line = Line::new(coord! { x: 8.0, y: 56.0 }, coord! { x: 8.2, y: 56.2 });
        /*let coords: Vec<CoordM<4326>> = [(1.0, 2.0, 0.0), (5.0, 3.0, 1.0), (3.0, 4.0, 2.0)]
            .map(|f| f.into())
            .to_vec();
        let first_line = LineM::from((coords[0],coords[1]));
        */

        let a = line_to_aabb_triangles(&line, 1.0,1.0,10000000.0,10000000.0);
        dbg!(a);
    }

    #[test]
    fn dumber_test() {
        use geo::Point;
        use linesonmaps::types::pointm::PointM;
use geo::{Distance, Geodesic};

// New York City
let new_york_city = PointM::<4326>::from((-74.006, 40.7128, 1.0));

// London
let london = PointM::<4326>::from((-0.1278, 51.5074, -1.0));

let distance = Geodesic.distance(new_york_city, london);
dbg!(distance);

assert_eq!(
    5_585_234., // meters
    distance.round()
);
    }
}
