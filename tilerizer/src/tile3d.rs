use crate::{Point, PointWTime, PointWZ};
use crate::{Zoom, point_to_grid};
use modeling::modeling::LineTriangle;
use std::cmp;

pub struct Triangle {
    pub v1: Point,
    pub v2: Point,
    pub v3: Point,
}

impl Triangle {
    pub fn get_bbox(&self) -> (i32, i32, i32, i32) {
        let bbminx = cmp::min(cmp::min(self.v1.x, self.v2.x), self.v3.x);
        let bbminy = cmp::min(cmp::min(self.v1.y, self.v2.y), self.v3.y);
        let bbmaxx = cmp::max(cmp::max(self.v1.x, self.v2.x), self.v3.x);
        let bbmaxy = cmp::max(cmp::max(self.v1.y, self.v2.y), self.v3.y);

        (bbminx, bbminy, bbmaxx, bbmaxy)
    }
}

pub fn draw_line_triangle(triangle: LineTriangle<4326>, sample_zoom_level: i32) -> Vec<PointWTime> {
    let triangle_grid = real_to_grid(&triangle.triangle, sample_zoom_level);
    let (bbminx, bbminy, bbmaxx, bbmaxy) = triangle_grid.get_bbox();
    let Triangle { v1, v2, v3 } = triangle_grid;
    let size = (bbmaxx - bbminx) * (bbmaxy - bbminy);

    let mut points: Vec<PointWTime> = Vec::with_capacity(size as usize / 2 + 1);

    let total_area = signed_total_area(v1.x, v1.y, v2.x, v2.y, v3.x, v3.y);

    for x in bbminx..=bbmaxx {
        for y in bbminy..=bbmaxy {
            let alpha = signed_total_area(x, y, v2.x, v2.y, v3.x, v3.y) / total_area;
            let beta = signed_total_area(x, y, v3.x, v3.y, v1.x, v1.y) / total_area;
            let gamma = signed_total_area(x, y, v1.x, v1.y, v2.x, v2.y) / total_area;
            if alpha >= 0. && beta >= 0. && gamma >= 0. {
                let timestamp = triangle.point_occupation(alpha, beta, gamma);
                let point = Point { x, y };
                points.push(PointWTime {
                    point,
                    z: sample_zoom_level,
                    time_start: timestamp.0,
                    time_end: timestamp.1,
                });
            }
        }
    }
    points
}

pub fn draw_triangle(triangle: geo_types::Triangle, sample_zoom_level: i32) -> Vec<PointWZ> {
    let triangle_grid = real_to_grid(&triangle, sample_zoom_level);
    let (bbminx, bbminy, bbmaxx, bbmaxy) = triangle_grid.get_bbox();
    let Triangle { v1, v2, v3 } = triangle_grid;
    let size = (bbmaxx - bbminx) * (bbmaxy - bbminy);

    let mut points: Vec<PointWZ> = Vec::with_capacity(size as usize / 2 + 1);

    let total_area = signed_total_area(v1.x, v1.y, v2.x, v2.y, v3.x, v3.y);

    for x in bbminx..=bbmaxx {
        for y in bbminy..=bbmaxy {
            let alpha = signed_total_area(x, y, v2.x, v2.y, v3.x, v3.y) / total_area;
            let beta = signed_total_area(x, y, v3.x, v3.y, v1.x, v1.y) / total_area;
            let gamma = signed_total_area(x, y, v1.x, v1.y, v2.x, v2.y) / total_area;
            if alpha >= 0. && beta >= 0. && gamma >= 0. {
                let point = Point { x, y };
                points.push(PointWZ {
                    point: point,
                    z: sample_zoom_level,
                });
            }
        }
    }
    points
}

fn real_to_grid(triangle: &geo_types::Triangle, sampling_zoom_level: i32) -> Triangle {
    Triangle {
        v1: point_to_grid(triangle.0, sampling_zoom_level),
        v2: point_to_grid(triangle.1, sampling_zoom_level),
        v3: point_to_grid(triangle.2, sampling_zoom_level),
    }
}

pub fn signed_total_area(v1x: i32, v1y: i32, v2x: i32, v2y: i32, v3x: i32, v3y: i32) -> f64 {
    0.5 * ((v2y - v1y) as i64 * (v2x + v1x) as i64
        + (v3y - v2y) as i64 * (v3x + v2x) as i64
        + (v1y - v3y) as i64 * (v1x + v3x) as i64) as f64
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use geo_types::{Line, Triangle};
    use linesonmaps::types::{linem::LineM, pointm::PointM};
    use modeling::modeling::line_to_triangle_pair;

    use super::*;

    #[test]
    fn draw_triangle_test() {
        // 57.01534956,9.99105250
        // 57.01322067,9.99096883

        let start_m =
            DateTime::parse_from_str("2024-01-01 00:00:00 +0000", "%Y-%m-%d %H:%M:%S%.3f %z")
                .unwrap()
                .timestamp() as f64;
        let end_m =
            DateTime::parse_from_str("2024-01-01 00:02:00 +0000", "%Y-%m-%d %H:%M:%S%.3f %z")
                .unwrap()
                .timestamp() as f64;

        let coord_1: PointM = (9.99105250, 57.01534956, start_m).into();
        let coord_2: PointM = (9.99096883, 57.01322067, end_m).into();
        let line = LineM::<4326>::from((coord_1, coord_2));
        let (a, b) = line_to_triangle_pair(&line, 50.0, 50.0, 50.0, 50.0);
        let result = draw_line_triangle(a, 20);
        let result_b = draw_line_triangle(b, 20);

        assert_eq!(result.len(), 35);
        assert_eq!(result_b.len(), 40);
    }
}
