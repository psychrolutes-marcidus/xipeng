use crate::point_to_grid;
use crate::{Point, PointWTime};
use geo_types::Coord;
use std::cmp;

pub struct RealWorldTriangle {
    pub v1: Coord<f64>,
    pub v2: Coord<f64>,
    pub v3: Coord<f64>,
}

pub struct Triangle {
    pub v1: Point,
    pub v2: Point,
    pub v3: Point,
}

impl RealWorldTriangle {
    pub fn into_triangle(&self, sampling_zoom_level: i32) -> Triangle {
        let v1 = point_to_grid(self.v1, sampling_zoom_level);
        let v2 = point_to_grid(self.v2, sampling_zoom_level);
        let v3 = point_to_grid(self.v3, sampling_zoom_level);

        Triangle { v1, v2, v3 }
    }
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

pub fn draw_triangle(triangle: Triangle, sample_zoom_level: i32) -> Vec<PointWTime> {
    let (bbminx, bbminy, bbmaxx, bbmaxy) = triangle.get_bbox();
    let Triangle {
        mut v1,
        mut v2,
        mut v3,
    } = triangle;
    let size = (bbmaxx - bbminx) * (bbmaxy - bbminy);
    if v1.y > v2.y {
        std::mem::swap(&mut v1, &mut v2);
    }
    if v1.y > v3.y {
        std::mem::swap(&mut v1, &mut v3);
    }
    if v2.y > v3.y {
        std::mem::swap(&mut v2, &mut v3);
    }

    let mut points: Vec<PointWTime> = Vec::with_capacity(size as usize / 2 + 1);

    let total_area = signed_total_area(v1.x, v1.y, v2.x, v2.y, v3.x, v3.y);

    for x in bbminx..=bbmaxx {
        for y in bbminy..=bbmaxy {
            let alpha = signed_total_area(x, y, v2.x, v2.y, v3.x, v3.y) / total_area;
            let beta = signed_total_area(x, y, v3.x, v3.y, v1.x, v1.y) / total_area;
            let gamma = signed_total_area(x, y, v1.x, v1.y, v2.x, v2.y) / total_area;
            if alpha >= 0. && beta >= 0. && gamma >= 0. {
                // TODO: Get timestamp here from alpha, beta and gamma
                let point = Point { x, y };
                points.push(PointWTime {
                    point,
                    z: sample_zoom_level,
                    time_stamps: Vec::new(),
                });
            }
        }
    }
    points
}

pub fn signed_total_area(v1x: i32, v1y: i32, v2x: i32, v2y: i32, v3x: i32, v3y: i32) -> f64 {
    0.5 * ((v2y - v1y) as i64 * (v2x + v1x) as i64
        + (v3y - v2y) as i64 * (v3x + v2x) as i64
        + (v1y - v3y) as i64 * (v1x + v3x) as i64) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_triangle_test() {
        let tri = RealWorldTriangle {
            v1: (9.9908885, 57.0131334).into(),
            v2: (9.9919149, 57.0123712).into(),
            v3: (9.9900846, 57.0117842).into(),
        };
        let result = draw_triangle(tri.into_triangle(20), 0);

        assert_eq!(result.len(), 17);
    }
}
