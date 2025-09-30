use chrono::prelude::*;
use data::tables::Ships;
use itertools::Itertools;
use linesonmaps::types::{coordm::CoordM, linestringm::LineStringM};
use std::cmp;

const KNOT_TO_MPS: f64 = 0.514444;
const SEC_TO_MILLISEC: f64 = 1000.0;

pub const SINGLE_VESSEL: u64 = 0;
pub const MULTI_VESSEL: u64 = 1;

#[derive(Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq)]
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
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

pub fn draw_linestring(ls: LineStringM<3857>, zoom_level: i32, sampling_zoom_level: i32) -> Vec<Tile> {


    // ls.lines().map(|x| )

    todo!()
}


pub fn point_to_grid(point: CoordM<4326>, sampling_zoom_level: i32) -> Point {
    use std::f64::consts::*;

    let x = (1./TAU*2_f64.powi(sampling_zoom_level)*(PI + (point.x * PI / 180.))).floor() as i32;
    let y = (1./TAU*2_f64.powi(sampling_zoom_level)*(PI - ((FRAC_PI_4+(point.y * PI/180.)/2.).tan()).ln())).floor() as i32;

    Point { x, y }

}

pub fn points_to_tiles(
    points: Vec<Point>,
    point_from: Point,
    point_to: Point,
    zoom_level: i32,
    draught: Option<i32>,
    time_from: DateTime<Utc>,
    time_to: DateTime<Utc>,
    sog: Option<f32>,
    length: Option<u32>,
    width: Option<u32>,
) -> Vec<Tile> {
    let delta_point = point_to - point_from;

    let max_tiles_dist = std::cmp::max(delta_point.x, delta_point.y);

    let mut time_delta = point_time_duration(time_from, time_to, max_tiles_dist);

    let time_delta_speed = length
        .zip(sog)
        .and_then(|(l, s)| Some((l as f64 / (s as f64 * KNOT_TO_MPS)) * SEC_TO_MILLISEC))
        .unwrap_or(0.);

    time_delta += chrono::TimeDelta::milliseconds(time_delta_speed as i64);

    let tiles: Vec<Tile> = points
        .into_iter()
        .map(|p| Tile {
            x: p.x,
            y: p.y,
            z: zoom_level,
            max_draught: draught,
            distinct_ship_count: 1,
            min_sog: sog,
            max_sog: sog,
            cell_oc_time: time_delta,
            min_length: length,
            max_length: length,
            min_width: width,
            max_width: width,
        })
        .collect();

    tiles
}

pub fn combine_tiles<const C: u64>(mut tiles: Vec<Tile>) -> Vec<Tile> {
    tiles.sort_unstable_by(|a, b| (a.x, a.y).cmp(&(b.x, b.y)));

    tiles
        .chunk_by(|a, b| a.x == b.x && a.y == b.y)
        .map(|tiledups| {
            tiledups
                .to_owned()
                .into_iter()
                .reduce(|acc, t| combine_tile::<C>(acc, t))
        })
        .filter_map(|x| x)
        .collect()
}

pub fn combine_tile<const C: u64>(a: Tile, b: Tile) -> Tile {
    Tile {
        max_draught: cmp::max(a.max_draught, b.max_draught),
        distinct_ship_count: a.distinct_ship_count + b.distinct_ship_count * C,
        min_sog: a.min_sog.map_or(b.min_sog, |ams| {
            b.min_sog.map_or(Some(ams), |bms| Some(ams.min(bms)))
        }),
        max_sog: a.max_sog.map_or(b.max_sog, |ams| {
            b.max_sog.map_or(Some(ams), |bms| Some(ams.max(bms)))
        }),
        cell_oc_time: a.cell_oc_time + b.cell_oc_time,
        min_length: cmp::min(a.min_length, b.min_length),
        max_length: cmp::max(a.max_length, b.max_length),
        min_width: cmp::min(a.min_width, b.min_width),
        max_width: cmp::max(a.max_width, b.max_width),
        ..a
    }
}

pub fn point_time_duration(
    time_from: DateTime<Utc>,
    time_to: DateTime<Utc>,
    point_count: i32,
) -> chrono::TimeDelta {
    let dt = time_to.signed_duration_since(time_from);

    let duration = dt.checked_div(point_count).unwrap_or(dt);

    return duration;
}


pub fn draw_line(from: Point, to: Point) -> Vec<Point> {
    let mut coordinates: Vec<Point> = vec![];
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();

    let sx = { if from.x < to.x { 1 } else { -1 } };
    let sy = { if from.y < to.y { 1 } else { -1 } };

    let mut error = (if dx > dy { dx } else { -dy }) / 2;
    let mut current_x = from.x;
    let mut current_y = from.y;

    loop {
        coordinates.push(Point {
            x: current_x,
            y: current_y,
            ..from
        });

        if current_x == to.x && current_y == to.y {
            break;
        }

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

        points.append(&mut draw_line(Point { x: 1, y: 1 }, Point { x: 3, y: 3 }));

        dbg!(&points);

        assert!(points[1].x == 2 && points[1].y == 2);
    }

    #[test]
    fn not_a_line() {
        let mut points: Vec<Point> = Vec::new();

        points.append(&mut draw_line(Point { x: 0, y: 0 }, Point { x: 0, y: 0 }));

        assert_eq!(points.len(), 1)
    }

    #[test]
    fn combine_tiles_same_coord() {
        let tiles = vec![
            Tile {
                x: 0,
                y: 0,
                z: 10,
                max_draught: None,
                min_sog: Some(1.0),
                max_sog: None,
                distinct_ship_count: 1,
                cell_oc_time: chrono::TimeDelta::seconds(5),
                min_length: Some(5),
                max_length: Some(5),
                min_width: None,
                max_width: Some(2),
            },
            Tile {
                x: 0,
                y: 0,
                z: 10,
                max_draught: Some(6),
                min_sog: Some(2.0),
                max_sog: Some(2.0),
                distinct_ship_count: 1,
                cell_oc_time: chrono::TimeDelta::seconds(5),
                min_length: Some(2),
                max_length: Some(2),
                min_width: None,
                max_width: Some(4),
            },
        ];

        let result = combine_tiles::<SINGLE_VESSEL>(tiles);

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            Tile {
                x: 0,
                y: 0,
                z: 10,
                max_draught: Some(6),
                min_sog: Some(1.0),
                max_sog: Some(2.0),
                distinct_ship_count: 1,
                cell_oc_time: chrono::TimeDelta::seconds(10),
                min_length: Some(2),
                max_length: Some(5),
                min_width: None,
                max_width: Some(4)
            }
        );
    }

    #[test]
    fn combine_tiles_diff_coord() {
        let tiles = vec![
            Tile {
                x: 0,
                y: 0,
                z: 10,
                max_draught: None,
                min_sog: Some(1.0),
                max_sog: None,
                distinct_ship_count: 1,
                cell_oc_time: chrono::TimeDelta::seconds(5),
                min_length: Some(5),
                max_length: Some(5),
                min_width: None,
                max_width: Some(2),
            },
            Tile {
                x: 1,
                y: 0,
                z: 10,
                max_draught: Some(6),
                min_sog: Some(2.0),
                max_sog: Some(2.0),
                distinct_ship_count: 1,
                cell_oc_time: chrono::TimeDelta::seconds(5),
                min_length: Some(2),
                max_length: Some(2),
                min_width: None,
                max_width: Some(4),
            },
        ];

        let result = combine_tiles::<SINGLE_VESSEL>(tiles);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn combine_tiles_diff_ships() {
        let tiles = vec![
            Tile {
                x: 0,
                y: 0,
                z: 10,
                max_draught: None,
                min_sog: Some(1.0),
                max_sog: None,
                distinct_ship_count: 1,
                cell_oc_time: chrono::TimeDelta::seconds(5),
                min_length: Some(5),
                max_length: Some(5),
                min_width: None,
                max_width: Some(2),
            },
            Tile {
                x: 0,
                y: 0,
                z: 10,
                max_draught: Some(6),
                min_sog: Some(2.0),
                max_sog: Some(2.0),
                distinct_ship_count: 1,
                cell_oc_time: chrono::TimeDelta::seconds(5),
                min_length: Some(2),
                max_length: Some(2),
                min_width: None,
                max_width: Some(4),
            },
        ];

        let result = combine_tiles::<MULTI_VESSEL>(tiles);

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            Tile {
                x: 0,
                y: 0,
                z: 10,
                max_draught: Some(6),
                min_sog: Some(1.0),
                max_sog: Some(2.0),
                distinct_ship_count: 2,
                cell_oc_time: chrono::TimeDelta::seconds(10),
                min_length: Some(2),
                max_length: Some(5),
                min_width: None,
                max_width: Some(4)
            }
        );
    }

    #[test]
    fn coord_to_point() {
        let cass_point = Point {x: 34586, y: 20073}; // At zoom 16
        let cass_4326_coord = CoordM::<4326> {x: 9.99083572, y: 57.01233944, m: 69.0};

        let result = point_to_grid(cass_4326_coord, 16);

        assert_eq!(cass_point, result);
    }
}
