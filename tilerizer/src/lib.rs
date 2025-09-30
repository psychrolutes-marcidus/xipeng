use linesonmaps::types::{linestringm::LineStringM, coordm::CoordM};
use data::tables::ship_draught::Draught;
use itertools::Itertools;
use chrono::prelude::*;

const KNOT_TO_MPS: f64 = 0.514444;
const SEC_TO_MILLISEC: f64 = 1000.0;


#[derive(Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub max_draught: Option<i32>,
    pub distinct_ship_count: u64,
    pub min_sog: Option<f32>,
    pub max_sog: Option<f32>,
    pub cell_oc_time: chrono::TimeDelta,
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    pub min_width: Option<u32>,
    pub max_width: Option<u32>,
}

pub trait Zoom {
    fn change_zoom(&mut self, zoom_level: i32);
}

impl Zoom for Tile {
fn change_zoom(&mut self, zoom_level: i32) {
        let change = self.z - zoom_level;

        if change > 0 {
            self.x /= 4_i32.pow(change as u32);
            self.y /= 4_i32.pow(change as u32);
        } else {
            self.x *= 4_i32.pow(change as u32);
            self.y *= 4_i32.pow(change as u32);
        }
        self.z = zoom_level;
    }
}

impl std::ops::Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {x: self.x - rhs.x, y: self.y - rhs.y}
    }
}

pub fn points_to_tiles(points: Vec<Point>, point_from: Point, point_to: Point, zoom_level: i32, draught: Option<i32>, time_from: DateTime<Utc>, time_to: DateTime<Utc>, sog: Option<f32>, length: Option<u32>, width: Option<u32>) -> Vec<Tile> {
    let delta_point = point_to - point_from;

    let max_tiles_dist = std::cmp::max(delta_point.x, delta_point.y);

    let mut time_delta = point_time_duration(time_from, time_to, max_tiles_dist);

    let sog_mps = sog.and_then(|s| Some(s*KNOT_TO_MPS));

    let time_delta_speed = length.zip(sog).and_then(|(l, s)| Some((l as f64 / (s as f64 *KNOT_TO_MPS)) * SEC_TO_MILLISEC)).unwrap_or(0.);

    time_delta += chrono::TimeDelta::milliseconds(time_delta_speed as i64);



    // points.into_iter().map(|p| Tile {x: p.x, y: p.y, z: }).collect()

    todo!()
}

pub fn point_time_duration(time_from: DateTime<Utc>, time_to: DateTime<Utc>, point_count: i32) -> chrono::TimeDelta {
    let dt = time_to.signed_duration_since(time_from);

    let duration = dt.checked_div(point_count).unwrap_or(dt);

    return duration;
}

// pub fn combine_tile_single_vessel(mut tiles:Vec<Tile>) -> Vec<Tile> {
//     tiles.sort_by_cached_key(|a| (a.x, a.y));

//     let something = tiles.chunk_by(|a, b| a.x == b.x && a.y == b.y).map(|tiledups| tiledups.into_iter().reduce(|acc, t| &Tile { draught: std::cmp::max(acc.draught, acc.draught), ..acc }));

//     todo!()
// }

// Should contain a unique Draught for that MMSI to be drawn.
// pub fn draw_linestring(ls: LineStringM<3857>, draught: Draught) {
//     ls.0.into_iter().map(f)
// }


// pub fn coord_to_tile(coord: CoordM<3857>, draught: &Draught) -> Tile {

// }

pub fn draw_line(from: Point, to: Point) -> Vec<Point> {
    let mut coordinates: Vec<Point> = vec![];
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();

    let sx = { if from.x < to.x { 1 } else { -1 }};
    let sy = { if from.y < to.y { 1 } else { -1 }};

    let mut error = (if dx > dy { dx } else {-dy }) / 2;
    let mut current_x = from.x;
    let mut current_y = from.y;

    loop {
        coordinates.push(Point { x: current_x, y: current_y, ..from });

        if current_x == to.x && current_y == to.y {break;}

        let error2 = error;

        if error2 > -dx {
            error -= dy;
            current_x += sx;
        }
        if error2 < dy {
            error += dx;
            current_y += sy;
        }
    }

    coordinates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line() {
        let mut points: Vec<Point> = Vec::new();

        points.append(&mut draw_line(Point { x: 1, y: 1, z: 22 }, Point { x: 3, y: 3, z: 22 }));

        dbg!(&points);

        assert!(points[1].x == 2 && points[1].y == 2);

    }

    #[test]
    fn not_a_line() {
        let mut points: Vec<Point> = Vec::new();

        points.append(&mut draw_line(Point { x: 0, y: 0, z: 22 }, Point { x: 0, y: 0, z: 22 }));

        assert_eq!(points.len(), 1)

    }
}
