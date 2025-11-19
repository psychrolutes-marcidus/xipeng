use std::num::NonZero;

use chrono::{Datelike, Timelike};
use linesonmaps::types::{coordm::CoordM, linem::LineM, linestringm::LineStringM, pointm::PointM};
use modeling::modeling::line_to_triangle_pair;
use pgrx::prelude::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tilerizer::{draw_linestring, point_to_grid, tile3d::draw_triangle, PointWTime, Zoom};
use wkb::reader::Dimension;

#[derive(Clone, Default, PostgresType, Serialize, Deserialize, AggregateName)]
pub struct RenderPointsAgg {
    points: Vec<PointWTime>,
}

#[pg_extern(parallel_safe, immutable)]
fn render_geom(
    geom: &[u8],
    zoom_level: i32,
    sampling_zoom_level: i32,
    a: Option<f64>,
    b: Option<f64>,
    c: Option<f64>,
    d: Option<f64>,
) -> TableIterator<
    'static,
    (
        name!(x, i32),
        name!(y, i32),
        name!(z, i32),
        name!(time_start, TimestampWithTimeZone),
        name!(time_end, TimestampWithTimeZone),
    ),
> {
    let geom = wkb::reader::read_wkb(&geom).expect("Could not read wkb");
    if geom.dimension() != Dimension::Xym {
        panic!("Received non XYM dimension geometry. It will be ignored");
    }

    let points = match geom.geometry_type() {
        wkb::reader::GeometryType::Point => {
            let pointm: PointM<4326> =
                PointM::try_from(CoordM::try_from(geom).expect("Expected a PointM"))
                    .expect("Expected a PointM");
            vec![render_point(pointm, sampling_zoom_level).change_zoom(zoom_level)]
        }
        wkb::reader::GeometryType::LineString => {
            let linem: LineStringM<4326> =
                LineStringM::try_from(geom).expect("Expected a LinestringM");
            let values = a.zip(b.zip(c.zip(d))).map(|(a, (b, (c, d)))| (a, b, c, d));
            match values {
                Some((a, b, c, d)) => {
                    let mut points: Vec<_> = linem
                        .lines()
                        .map(|line: LineM<4326>| line_to_triangle_pair(&line, a, b, c, d))
                        .flat_map(|(tri1, tri2)| {
                            [
                                draw_triangle(tri1, sampling_zoom_level),
                                draw_triangle(tri2, sampling_zoom_level),
                            ]
                        })
                        .flatten()
                        .map(|x| x.change_zoom(zoom_level))
                        .collect();
                    points.par_sort_by_key(|p| (p.point, p.time_start, p.time_end));
                    points
                        .par_chunk_by(|a, b| a.point == b.point && a.time_end >= b.time_start)
                        .map(|p| {
                            let first = p.first().expect("Chunks are not empty");
                            let last = p.last().expect("Chunks are not empty");
                            PointWTime {
                                time_end: last.time_end,
                                ..*first
                            }
                        })
                        .collect()
                }
                None => draw_linestring(linem, zoom_level, sampling_zoom_level),
            }
        }
        wkb::reader::GeometryType::Polygon => todo!(),
        wkb::reader::GeometryType::MultiPoint => todo!(),
        wkb::reader::GeometryType::MultiLineString => todo!(),
        wkb::reader::GeometryType::MultiPolygon => todo!(),
        wkb::reader::GeometryType::GeometryCollection => todo!(),
        _ => todo!(),
    };
    let points: Vec<_> = points
        .into_iter()
        .map(|p| {
            (
                p.point.x,
                p.point.y,
                p.z,
                TimestampWithTimeZone::new(
                    p.time_start.year() as i32,
                    p.time_start.month() as u8,
                    p.time_start.day() as u8,
                    p.time_start.hour() as u8,
                    p.time_start.minute() as u8,
                    p.time_start.second() as f64
                        + (p.time_start.nanosecond() as f64 / 1000000000.0),
                )
                .unwrap(),
                TimestampWithTimeZone::new(
                    p.time_end.year() as i32,
                    p.time_end.month() as u8,
                    p.time_end.day() as u8,
                    p.time_end.hour() as u8,
                    p.time_end.minute() as u8,
                    p.time_end.second() as f64 + (p.time_end.nanosecond() as f64 / 1000000000.0),
                )
                .unwrap(),
            )
        })
        .collect();
    TableIterator::new(points)
}

#[pg_aggregate(parallel_safe)]
impl Aggregate<RenderPointsAgg> for RenderPointsAgg {
    const INITIAL_CONDITION: Option<&'static str> = Some(r#"{ "points": [] }"#);
    type Args = (Vec<u8>, i32, i32);
    fn state(
        mut current: Self::State,
        (line, zoom, samp_zoom): Self::Args,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
        let line_wkb = wkb::reader::read_wkb(&line).expect("Could not read wkb");
        if line_wkb.dimension() != Dimension::Xym {
            notice!("Received non XYM dimension geometry. It will be ignored for the aggregate function.");
            return current;
        }

        let points: Vec<PointWTime> = match line_wkb.geometry_type() {
            wkb::reader::GeometryType::Point => {
                let pointm: PointM<4326> =
                    PointM::try_from(CoordM::try_from(line_wkb).expect("Expected a PointM"))
                        .expect("Nothing");
                vec![render_point(pointm, samp_zoom)]
            }
            wkb::reader::GeometryType::LineString => todo!(),
            wkb::reader::GeometryType::Polygon => todo!(),
            _ => panic!(),
        };
        current.points.extend_from_slice(&points);
        current
    }
}

fn render_point(point: PointM, sampling_zoom_level: i32) -> PointWTime {
    let grid_point = point_to_grid((point.coord.x, point.coord.y).into(), sampling_zoom_level);
    PointWTime {
        point: grid_point,
        z: sampling_zoom_level,
        time_start: chrono::DateTime::from_timestamp_secs(point.coord.m as i64)
            .expect("Should work"),
        time_end: chrono::DateTime::from_timestamp_secs(point.coord.m as i64).expect("Should work"),
    }
}
