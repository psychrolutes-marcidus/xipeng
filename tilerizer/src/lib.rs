use chrono::prelude::*;
use data::tables::Ships;
use itertools::Itertools;
use linesonmaps::types::{coordm::CoordM, linestringm::LineStringM};
use std::cmp;

const KNOT_TO_MPS: f64 = 0.514444;
const SEC_TO_MILLISEC: f64 = 1000.0;

pub const SINGLE_VESSEL: u64 = 0;
pub const MULTI_VESSEL: u64 = 1;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PointWTime {
    pub point: Point,
    pub time_stamps: Vec<(DateTime<Utc>, DateTime<Utc>)>,
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

pub trait Combineable<T> {
    fn combine(&self, other: T) -> Self;
}

// impl Combineable<PointWTime> for PointWTime {
//     fn combine(&self, other: PointWTime) -> Self {
//         Self { time_stamps: () }

//         todo!()
//     }
// }

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

pub fn draw_linestring2(ls: LineStringM<4326>, zoom_level: i32, sampling_zoom_level: i32) -> Vec<PointWTime> {
    let point_ext: Vec<Vec<PointWTime>> = ls.points().map(|p| (point_to_grid(p.coord, sampling_zoom_level), DateTime::from_timestamp_secs(p.coord.m as i64).unwrap())).tuple_windows().map(|((ap,at),(bp, bt))| interpolate_time_to_point(draw_line(ap, bp), at, bt)).collect();

    // Reduce after this.

    todo!()
}

pub fn interpolate_time_to_point(points: Vec<Point>, time_from: DateTime<Utc>, time_to: DateTime<Utc>) -> Vec<PointWTime> {
    let dtime = time_to - time_from;

    let len = points.len();

    if len < 1 {
        return Vec::new();
    }

    let dtime =  dtime / (len as i32);

    points.into_iter().enumerate().map(|(i, p)| PointWTime {point: p, time_stamps: vec![(std::cmp::max(time_from, time_from + dtime * i as i32 - dtime / 2), std::cmp::min(time_to, time_from + dtime * i as i32 + dtime / 2 ))] } ).collect()


}

// First iteration
pub fn draw_linestring(
    ls: LineStringM<4326>,
    mmsi: i32,
    ships: &data::tables::Ships,
    zoom_level: i32,
    sampling_zoom_level: i32,
) -> Vec<Tile> {
    let time_start =
        DateTime::from_timestamp_secs(ls.0.first().map(|x| x.m).unwrap() as i64).unwrap();
    let time_end = DateTime::from_timestamp_secs(ls.0.last().map(|x| x.m).unwrap() as i64).unwrap();

    let draughts: Vec<(DateTime<Utc>, DateTime<Utc>, f32)> = itertools::izip!(
        &ships.ship_draught.mmsi,
        &ships.ship_draught.time_begin,
        &ships.ship_draught.time_end,
        &ships.ship_draught.draught
    )
    .filter(|(m, ts, te, _)| **m == mmsi && **ts <= time_end && **te >= time_start)
    .map(|(_, ts, te, d)| (*ts, *te, *d))
    .collect();

    let sogs: Vec<(DateTime<Utc>, f32)> =
        itertools::izip!(&ships.sog.mmsi, &ships.sog.time, &ships.sog.sog)
            .filter(|(m, t, _)| **m == mmsi && **t <= time_end && **t >= time_start)
            .map(|(_, t, s)| (*t, *s))
            .collect();

    let (width, length) = itertools::izip!(
        &ships.dimensions.mmsi,
        &ships.dimensions.width,
        &ships.dimensions.length
    )
    .find(|(m, _, _)| **m == mmsi)
    .map(|(_, w, l)| (*w as u32, *l as u32))
    .unzip();

    let tiles: Vec<Tile> = ls
        .points()
        .map(|p| {
            (
                point_to_grid(p.coord, sampling_zoom_level),
                DateTime::from_timestamp_secs(p.coord.m as i64).unwrap(),
            )
        })
        .map(|(p, t)| {
            (
                p,
                t,
                draughts
                    .iter()
                    .find(|(ts, te, _)| *ts <= t && *te >= t)
                    .map(|(_, _, d)| *d as i32),
                sogs.iter().find(|(ts, _)| *ts == t).map(|(_, s)| s),
            )
        })
        .tuple_windows()
        .map(|(a, b)| {
            points_to_tiles(
                draw_line(a.0, b.0),
                a.0,
                b.0,
                sampling_zoom_level,
                a.2,
                a.1,
                b.1,
                a.3.copied(),
                length,
                width,
            )
        })
        .flatten().map(|mut p| {p.change_zoom(zoom_level); p})
        .collect();

    combine_tiles::<SINGLE_VESSEL>(tiles)
}

pub fn point_to_grid(point: CoordM<4326>, sampling_zoom_level: i32) -> Point {
    use std::f64::consts::*;

    let x =
        (1. / TAU * 2_f64.powi(sampling_zoom_level) * (PI + (point.x * PI / 180.))).floor() as i32;
    let y = (1. / TAU
        * 2_f64.powi(sampling_zoom_level)
        * (PI - ((FRAC_PI_4 + (point.y * PI / 180.) / 2.).tan()).ln()))
    .floor() as i32;

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
        let cass_point = Point { x: 34586, y: 20073 }; // At zoom 16
        let cass_4326_coord = CoordM::<4326> {
            x: 9.99083572,
            y: 57.01233944,
            m: 69.0,
        };

        let result = point_to_grid(cass_4326_coord, 16);

        assert_eq!(cass_point, result);
    }
}
