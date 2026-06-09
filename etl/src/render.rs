use std::collections::VecDeque;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use algorithms::cell::{gravity_model, judweight_vessel, st_tileenvelope};
use cached::TtlCache;
use cached::macros::cached;
use duckdb::params;
use duckdb::{Config, Connection, OptionalExt};
use fafo::util::ground_truth_to_cell_centroid_geodesic;
use fafo::xyzcell::Cell;
use fafo::{
    ErrorMeasurementConf, cells_relative_coverage_by_polygon,
    line_error_relative_to_perfect_and_centroid,
};
use geo::{Geometry, Intersects, Point};
use geo_traits::GeometryTrait;
use geo_traits::to_geo::{
    ToGeoGeometry, ToGeoLine, ToGeoLineString, ToGeoMultiPolygon, ToGeoPoint, ToGeoPolygon,
    ToGeoTriangle,
};
use linesonmaps::types::coordm::CoordM;
use linesonmaps::types::linem::LineM;
use linesonmaps::types::pointm::PointM;
use modeling::modeling::line_to_triangle_pair;
use rayon::prelude::*;
use rstar::primitives::{GeomWithData, Rectangle};
use rstar::{RTree, RTreeObject};
use sysinfo::System;

use crate::RenderCell;

#[derive(Debug, Clone, Copy)]
struct DbPoint {
    pub lon: f32,
    pub lat: f32,
    pub time: f32,
}

#[derive(Debug, Clone, Copy)]
struct DbDimensions {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}

#[derive(Debug, Clone, Copy)]
struct DbParameters {
    pub draught_dist_mmsi: f32,
    pub draught_dist_type: f32,
    pub draught_nulls: f32,
    pub r_squared: f32,
}

#[derive(Debug, Clone, Copy)]
struct DbTile {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

const EXTENSION_QUERY: &str = "LOAD '/home/rasmus/Projekter/xipeng/ferruginous/build/release/ferruginous.duckdb_extension'; LOAD spatial; SET geometry_always_xy = true;";

pub fn render_cells(params: RenderCell) {
    let config = Config::default()
        .allow_unsigned_extensions()
        .expect("Could not allow unsigned extensions");
    let con = Connection::open_with_flags(params.db_path.clone(), config)
        .expect("Could not open database");
    println!("Loading extension");
    con.execute_batch(EXTENSION_QUERY)
        .expect("Could not load extension");
    println!("Setup rendering views and tables");
    setup_rendering(&con, &params).expect("Could not setup rendering views and tables");
    println!("Getting candidate cells");
    con.close().expect("Could not close connection");
    let config = Config::default()
        .enable_autoload_extension(true)
        .expect("Cannot autoload extensions")
        .allow_unsigned_extensions()
        .expect("Cannot allow unsigned extensions")
        .access_mode(duckdb::AccessMode::ReadOnly)
        .expect("Cannot open in read only mode");
    let con = Connection::open_with_flags(params.db_path.clone(), config)
        .expect("Could not open connection pool");
    con.execute_batch(EXTENSION_QUERY)
        .expect("Could not load extensions");
    let candidate_cells =
        get_candidate_cells(con, &params).expect("Could not receive candidate cells");

    let config = Config::default()
        .allow_unsigned_extensions()
        .expect("Could not allow unsigned extensions");
    let con = Connection::open_with_flags(params.db_path.clone(), config)
        .expect("Could not open database");
    println!("Loading extension");
    con.execute_batch(EXTENSION_QUERY)
        .expect("Could not load extension");
    println!("Rendering cells to table");
    render_cell_to_table(&con, &candidate_cells, &params).expect("Could not render cells to table");
}

pub fn setup_rendering(
    tx: &Connection,
    params: &RenderCell,
) -> Result<(), Box<dyn std::error::Error>> {
    tx.execute_batch(
        "CREATE OR REPLACE TABLE draught_dist_mmsi_normal AS (
  SELECT *
  FROM render.draught_dist_mmsi_normal
);
CREATE OR REPLACE TABLE draught_dist_vessel_type_normal AS (
  SELECT *
  FROM render.draught_dist_vessel_type_normal
);
CREATE OR REPLACE TABLE draught_nulls_by_ship_type AS (
  SELECT *
  FROM vessel_attributes.draught_nulls_by_ship_type
);",
    )?;
    let query_start = "CREATE OR REPLACE VIEW trajs AS (
      SELECT
        ap.mmsi,
        ap.timestamp,
        {'lon': lon, 'lat': lat, 'time': epoch(ap.timestamp)} as point,
        CASE WHEN to_bow IS NOT NULL AND to_starboard IS NOT NULL AND to_stern IS NOT NULL AND to_port IS NOT NULL THEN {
              'to_bow': ap.to_bow::float,
              'to_starboard': ap.to_starboard::float,
              'to_stern': ap.to_stern::float,
              'to_port': ap.to_port::float
      } ELSE NULL
      END as dimensions,
      draught,
      ship_type
      FROM
        ais_data ap";
    let query = format!(
        "{}
        WHERE ap.timestamp >= '{}' AND ap.timestamp <= '{}'
        );",
        query_start, params.time_start, params.time_stop
    );
    tx.execute_batch(&query)?;

    tx.execute_batch(
        "CREATE OR REPLACE VIEW lines AS (
  SELECT
    ap.mmsi,
    ap.timestamp,
    ap.point,
    CASE
      WHEN LEAD (ap.timestamp) OVER (
      PARTITION BY mmsi
        ORDER BY
          ap.timestamp
      ) > ap.timestamp
      AND trajectory_split (
        ap.point,
        LEAD (ap.point, 1, NULL) OVER (
          PARTITION BY
            mmsi
          ORDER BY
            ap.timestamp
        )
      )
      AND (
        LEAD (ap.point) OVER (
          PARTITION BY
            mmsi
          ORDER BY
            ap.timestamp
        ).lat != ap.point.lat
        OR LEAD (ap.point) OVER (
          PARTITION BY
            mmsi
          ORDER BY
            ap.timestamp
        ).lon != ap.point.lon
      ) THEN LEAD (ap.point, 1, NULL) OVER (
        PARTITION BY mmsi
        ORDER BY
          ap.timestamp
      )
      ELSE NULL
    END AS next_point,
    dimensions,
    draught,
    ship_type
  FROM
    trajs ap
);
",
    )?;
    let sys = System::new_all();
    let threads = sys.cpus().len();

    println!("Polygonise lines");

    let parser = |x: &String| {
        x.split(",")
            .flat_map(|x| x.parse::<i32>())
            .take(3)
            .collect()
    };
    let tile_start: Vec<_> = parser(&params.tile_start);
    let tile_end: Vec<_> = parser(&params.tile_end.as_ref().unwrap_or(&params.tile_start));

    let rest = "
  SELECT
    ap.mmsi,
    ap.timestamp,
    ap.point,
    ap.next_point,
    CASE
      WHEN ap.next_point IS NOT NULL
      AND dimensions IS NOT NULL
      THEN st_geomfromwkb (polyganize (ap.point, ap.next_point, dimensions))
      WHEN ap.next_point IS NOT NULL
      AND dimensions IS NULL
      THEN ST_MakeLine (
        ST_Point (ap.point.lon, ap.point.lat),
        ST_Point (ap.next_point.lon, ap.next_point.lat)
      )
      ELSE ST_Point (ap.point.lon, ap.point.lat)
    END as geom,
    dimensions,
    ap.draught,
    ap.ship_type,
    ST_Area(geom) as area
  FROM
    lines ap
    WHERE (SELECT true FROM cand_cells WHERE ST_Intersects(cellgeom, geom) LIMIT 1)
);

-- CREATE INDEX geom_idx ON lines_with_geom USING RTREE (geom)";
    let sql = format!("LOAD spatial;
SET
  geometry_always_xy = TRUE;
CREATE TABLE IF NOT EXISTS lines_with_geom AS (
  WITH cand_cells AS MATERIALIZED (
SELECT
              xt.* as x,
              yt.* as y,
              {} as z,
              ST_Transform(ST_TileEnvelope (z::integer, x::integer, y::integer), 'EPSG:3857', 'EPSG:4326') as cellgeom
            FROM
              generate_series({}, {}, 1) xt,
              generate_series({}, {}, 1) yt
              )
              {}", tile_start[2], tile_start[0], tile_end[0], tile_start[1], tile_end[1], rest);
    tx.execute_batch(&sql)?;

    Ok(())
}

type RectIdx = GeomWithData<Rectangle<Point>, usize>;
fn search_tile(
    index: Arc<RwLock<RTree<RectIdx>>>,
    geom_list: Arc<RwLock<Vec<Geometry>>>,
    manager: &Connection,
    x: i32,
    y: i32,
    z: i32,
) {
    let wkb_row = manager.query_row("SELECT ST_AsWKB(ST_Transform(geom, 'EPSG:4326', 'EPSG:3857', always_xy := true)) FROM lines_with_geom WHERE ST_Intersects(ST_Transform(ST_TileEnvelope(?, ?, ?), 'EPSG:3857', 'EPSG:4326', always_xy := true), geom) ORDER BY area DESC LIMIT 1", [z, x, y], |row| (row.get::<_, Vec<u8>>(0))).optional().unwrap();
    match wkb_row {
        Some(w) => {
            let mut index = index.write().expect("Could not get write lock");
            let mut list = geom_list.write().expect("Could not get write lock");
            let geom = wkb::reader::read_wkb(&w).expect("Malformed wkb");
            match geom.as_type() {
                geo_traits::GeometryType::Point(p) => {
                    let index_geom =
                        RectIdx::new(Rectangle::from_aabb(p.to_point().envelope()), list.len());
                    index.insert(index_geom);
                    list.push(p.to_geometry());
                }
                geo_traits::GeometryType::LineString(ls) => {
                    let index_geom = RectIdx::new(
                        Rectangle::from_aabb(ls.to_line_string().envelope()),
                        list.len(),
                    );
                    index.insert(index_geom);
                    list.push(ls.to_geometry());
                }
                geo_traits::GeometryType::Polygon(p) => {
                    let index_geom =
                        RectIdx::new(Rectangle::from_aabb(p.to_polygon().envelope()), list.len());
                    index.insert(index_geom);
                    list.push(p.to_geometry());
                }
                geo_traits::GeometryType::Triangle(t) => {
                    let index_geom =
                        RectIdx::new(Rectangle::from_aabb(t.to_triangle().envelope()), list.len());
                    index.insert(index_geom);
                    list.push(t.to_geometry());
                }
                geo_traits::GeometryType::Line(l) => {
                    let index_geom =
                        RectIdx::new(Rectangle::from_aabb(l.to_line().envelope()), list.len());
                    index.insert(index_geom);
                    list.push(l.to_geometry());
                }
                _ => unimplemented!(),
            };
        }
        None => {}
    }
}

fn get_index(
    con: &Connection,
    x: i32,
    y: i32,
    z: i32,
    limit: i64,
) -> Result<
    (
        rstar::RTree<GeomWithData<Rectangle<Point>, usize>>,
        Vec<Geometry>,
        i64,
    ),
    Box<dyn std::error::Error>,
> {
    let sql = format!(
        "SELECT st_aswkb(st_transform(a.geom, 'EPSG:4326', 'EPSG:3857'))
FROM lines_with_geom a
WHERE ST_Intersects(st_transform(st_tileenvelope({}, {}, {}), 'EPSG:3857', 'EPSG:4326'), a.geom)
LIMIT {}",
        z, x, y, limit
    );
    let mut stmt = con.prepare(&sql)?;
    let wkbs: Vec<_> = stmt
        .query_map([], |row| row.get::<_, Vec<u8>>(0))?
        .map(|x| x.unwrap())
        .collect();

    let (aabbs, geoms): (Vec<_>, Vec<_>) = wkbs
        .par_iter()
        .enumerate()
        .map(|(i, geom)| {
            let geom = wkb::reader::read_wkb(&geom).expect("Malformed wkb");
            match geom.as_type() {
                geo_traits::GeometryType::Point(p) => (
                    RectIdx::new(
                        rstar::primitives::Rectangle::from_aabb(p.to_point().envelope()),
                        i,
                    ),
                    p.to_geometry(),
                ),
                geo_traits::GeometryType::Line(l) => (
                    RectIdx::new(
                        rstar::primitives::Rectangle::from_aabb(l.to_line().envelope()),
                        i,
                    ),
                    l.to_geometry(),
                ),
                geo_traits::GeometryType::Polygon(poly) => (
                    RectIdx::new(
                        rstar::primitives::Rectangle::from_aabb(poly.to_polygon().envelope()),
                        i,
                    ),
                    poly.to_geometry(),
                ),
                geo_traits::GeometryType::LineString(lines) => (
                    RectIdx::new(
                        rstar::primitives::Rectangle::from_aabb(lines.to_line_string().envelope()),
                        i,
                    ),
                    lines.to_geometry(),
                ),
                geo_traits::GeometryType::MultiPolygon(mp) => (
                    RectIdx::new(
                        rstar::primitives::Rectangle::from_aabb(mp.to_multi_polygon().envelope()),
                        i,
                    ),
                    mp.to_geometry(),
                ),
                geo_traits::GeometryType::Triangle(t) => (
                    RectIdx::new(
                        rstar::primitives::Rectangle::from_aabb(t.to_triangle().envelope()),
                        i,
                    ),
                    t.to_geometry(),
                ),
                _ => unimplemented!(),
            }
        })
        .unzip();
    drop(wkbs);
    let mut count = geoms.len() as i64;
    if count == limit {
        count = con
            .query_row(
                "SELECT count(*)
FROM lines_with_geom a
WHERE ST_Intersects(st_transform(st_tileenvelope(?, ?, ?), 'EPSG:3857', 'EPSG:4326'), a.geom)",
                [z, x, y],
                |row| row.get::<_, i64>(0),
            )
            .expect("Could not get count");
    }

    let index = rstar::RTree::bulk_load(aabbs);
    Ok((index, geoms, count))
}

type RenderTuple = Vec<(
    i32,
    String,
    f32,
    CoordM,
    Option<CoordM>,
    Option<DbDimensions>,
)>;
#[cached(
    ty = "TtlCache<String, (RenderTuple, Vec<Geometry>, rstar::RTree<GeomWithData<Rectangle<Point>, usize>>)>",
    create = "{ TtlCache::with_ttl_and_refresh(Duration::from_mins(5), true) }",
    convert = r#"{ format!("{},{},{}", x, y, z) }"#,
    sync_writes = "by_key"
)]
fn get_cell_data(
    con: &Connection,
    x: i32,
    y: i32,
    z: i32,
) -> (
    RenderTuple,
    Vec<Geometry>,
    rstar::RTree<GeomWithData<Rectangle<Point>, usize>>,
) {
    con.execute_batch(EXTENSION_QUERY)
        .expect("Should not happen");
    let mut stmt = con
        .prepare_cached(
            "WITH data_filtered AS materialized
    (SELECT *
     FROM lines_with_geom
     WHERE draught IS NOT NULL
         AND st_intersects(st_transform(st_tileenvelope(?, ?, ?), 'EPSG:3857', 'EPSG:4326'), geom))
SELECT mmsi,
       ship_type,
       draught::float,
       point.lon::DOUBLE,
       point.lat::DOUBLE,
       point.time,
       next_point.lon::DOUBLE,
       next_point.lat::DOUBLE,
       next_point.time,
       dimensions.to_bow,
       dimensions.to_starboard,
       dimensions.to_stern,
       dimensions.to_port,
       ST_Transform(geom, 'EPSG:4326', 'EPSG:3857') as geom
FROM data_filtered;",
        )
        .expect("Should not fail");
    let data: Vec<_> = stmt
        .query_map([z, x, y], |row| {
            Ok((
                row.get::<_, i32>(0).unwrap(),
                row.get::<_, String>(1).unwrap(),
                row.get::<_, f32>(2).unwrap(),
                CoordM::<4326> {
                    x: row.get::<_, f64>(3).unwrap(),
                    y: row.get::<_, f64>(4).unwrap(),
                    m: row.get::<_, f64>(5).unwrap(),
                },
                row.get::<_, f64>(6)
                    .ok()
                    .zip(row.get::<_, f64>(7).ok())
                    .zip(row.get::<_, f64>(8).ok())
                    .map(|((lon, lat), time)| CoordM::<4326> {
                        x: lon,
                        y: lat,
                        m: time,
                    }),
                row.get::<_, f32>(9)
                    .ok()
                    .zip(row.get::<_, f32>(10).ok())
                    .zip(row.get::<_, f32>(11).ok())
                    .zip(row.get::<_, f32>(12).ok())
                    .map(|(((a, b), c), d)| DbDimensions { a, b, c, d }),
                row.get::<_, Vec<u8>>(13).unwrap(),
            ))
        })
        .unwrap()
        .map(|x| x.expect("Remove all those results"))
        .collect();

    let (rect, geom): (Vec<_>, Vec<_>) = data
        .iter()
        .map(|x| &x.6)
        .enumerate()
        .map(|(i, bin)| {
            let geom = wkb::reader::read_wkb(&bin).expect("Malformed wkb");
            let (rect, geom) = match geom.as_type() {
                geo_traits::GeometryType::Point(p) => (
                    RectIdx::new(
                        rstar::primitives::Rectangle::from_aabb(p.to_point().envelope()),
                        i,
                    ),
                    p.to_geometry(),
                ),
                geo_traits::GeometryType::LineString(ls) => (
                    RectIdx::new(
                        rstar::primitives::Rectangle::from_aabb(ls.to_line_string().envelope()),
                        i,
                    ),
                    ls.to_geometry(),
                ),
                geo_traits::GeometryType::Polygon(p) => (
                    RectIdx::new(
                        rstar::primitives::Rectangle::from_aabb(p.to_polygon().envelope()),
                        i,
                    ),
                    p.to_geometry(),
                ),
                geo_traits::GeometryType::MultiPolygon(mp) => (
                    RectIdx::new(
                        rstar::primitives::Rectangle::from_aabb(mp.to_multi_polygon().envelope()),
                        i,
                    ),
                    mp.to_geometry(),
                ),
                geo_traits::GeometryType::Triangle(t) => (
                    RectIdx::new(
                        rstar::primitives::Rectangle::from_aabb(t.to_triangle().envelope()),
                        i,
                    ),
                    t.to_geometry(),
                ),
                geo_traits::GeometryType::Line(l) => (
                    RectIdx::new(
                        rstar::primitives::Rectangle::from_aabb(l.to_line().envelope()),
                        i,
                    ),
                    l.to_geometry(),
                ),
                _ => unimplemented!(),
            };
            (rect, geom)
        })
        .unzip();

    let index = rstar::RTree::bulk_load(rect);

    (
        data.iter()
            .map(|(mmsi, stype, draught, point, next_point, dimensions, _)| {
                (
                    *mmsi,
                    stype.clone(),
                    *draught,
                    *point,
                    *next_point,
                    *dimensions,
                )
            })
            .collect::<Vec<_>>(),
        geom,
        index,
    )
}

#[cached(
    ty = "TtlCache<String, Option<f32>>",
    create = "{ TtlCache::with_ttl_and_refresh(Duration::from_mins(5), true) }",
    convert = r#"{ format!("{}{}", mmsi, draught) }"#,
    sync_writes = "by_key"
)]
fn get_draught_dist_vessel_type_normal(con: &Connection, mmsi: i32, draught: f32) -> Option<f32> {
    let mut stmt = con
        .prepare_cached(
            "SELECT score_norm FROM draught_dist_vessel_type_normal WHERE mmsi = ? AND draught = ? AND draught IS NOT NULL",
        )
        .expect("Could not prepare statement");

    stmt.query_one(params![mmsi, draught], |row| row.get::<_, f32>(0))
        .optional()
        .expect("query went wrong")
}

#[cached(
    ty = "TtlCache<String, Option<f32>>",
    create = "{ TtlCache::with_ttl_and_refresh(Duration::from_mins(5), true) }",
    convert = r#"{ format!("{}{}", mmsi, draught) }"#,
    sync_writes = "by_key"
)]
fn get_draught_dist_mmsi_normal(con: &Connection, mmsi: i32, draught: f32) -> Option<f32> {
    let mut stmt = con.prepare_cached("SELECT score_norm FROM draught_dist_mmsi_normal WHERE mmsi = ? AND draught = ? AND draught IS NOT NULL").expect("Could not prepare statement");
    stmt.query_one(params![mmsi, draught], |row| row.get::<_, f32>(0))
        .optional()
        .expect("query went wrong")
}

#[cached(
    ty = "TtlCache<String, Option<f32>>",
    create = "{ TtlCache::with_ttl_and_refresh(Duration::from_mins(5), true) }",
    convert = r#"{ format!("{}", vessel_type) }"#,
    sync_writes = "by_key"
)]
fn get_linear_regression(con: &Connection, vessel_type: &str) -> Option<f32> {
    let mut stmt = con
        .prepare_cached("SELECT r_squared FROM vessel_stats.linear_regression WHERE ship_type = ?")
        .expect("Could not prepare statement");

    stmt.query_one([vessel_type], |row| row.get::<_, f32>(0))
        .optional()
        .expect("query went wrong")
}

#[cached(
    ty = "TtlCache<i32, Option<f32>>",
    create = "{ TtlCache::with_ttl_and_refresh(Duration::from_mins(5), true) }",
    convert = r#"{ mmsi }"#,
    sync_writes = "by_key"
)]
fn get_std_draught(con: &Connection, mmsi: i32) -> Option<f32> {
    let mut stmt = con
        .prepare_cached("SELECT median_draught FROM vessel_stats.std_draught WHERE mmsi = ?")
        .expect("Could not prepare statement");
    stmt.query_one([mmsi], |row| row.get::<_, f32>(0))
        .optional()
        .expect("query went wrong")
}

#[cached(
    ty = "TtlCache<String, Option<f32>>",
    create = "{ TtlCache::with_ttl_and_refresh(Duration::from_mins(5), true) }",
    convert = r#"{ format!("{}", vessel_type) }"#,
    sync_writes = "by_key"
)]
fn get_draught_nulls_by_ship_type(con: &Connection, vessel_type: &str) -> Option<f32> {
    let mut stmt = con
        .prepare_cached("SELECT draughts_null FROM draught_nulls_by_ship_type WHERE ship_type = ?")
        .expect("Could not prepare statement");
    stmt.query_one([vessel_type], |row| row.get::<_, f32>(0))
        .optional()
        .expect("query went wrong")
}

fn request_cell(con: &Connection, x: i32, y: i32, z: i32, z_limit: i32) -> RenderTuple {
    let diff = z - z_limit;
    let qx = x >> diff;
    let qy = y >> diff;
    let (items, geoms, rtree) = get_cell_data(con, qx, qy, z_limit);

    let tile = st_tileenvelope(z as u32, x, y);
    let inter_items = rtree
        .locate_in_envelope_intersecting(&tile.envelope())
        .filter_map(|x| match geoms[x.data].intersects(&tile) {
            true => Some(x.data),
            false => None,
        })
        .map(|x| items[x].to_owned())
        .collect();

    inter_items
}

pub fn get_candidate_cells(
    manager: Connection,
    params: &RenderCell,
) -> Result<Vec<(i32, i32)>, Box<dyn std::error::Error>> {
    // let a_manager = Arc::new(Mutex::new(manager));
    let parser = |x: &String| {
        x.split(",")
            .flat_map(|x| x.parse::<i32>())
            .take(3)
            .collect()
    };
    let tile_start: Vec<_> = parser(&params.tile_start);

    assert_eq!(tile_start.len(), 3);

    // le.map(|x| x.0.ok().zip(x.1.ok()).zip(x.2.okdraughtmapscorex| medt  (draught, score, medcurrent_z_diff = params.level - tile_start[2];
    // tile_start[0] = tile_start[0] * 2_i32.pow(current_z_diff as u32);
    // tile_start[1] = tile_start[1] * 2_i32.pow(current_z_diff as u32);
    // tile_start[2] = params.level;

    // let tile_ender = |tile_end: Vec<i32>| {
    //     let mut tile_end = tile_end.clone();
    //     let current_z_diff = params.level - tile_end[2];
    //     tile_end[0] = (tile_end[0] + 1) * 2_i32.pow(current_z_diff as u32);
    //     tile_end[1] = (tile_end[1] + 1) * 2_i32.pow(current_z_diff as u32);
    //     tile_end[2] = params.level;
    //     tile_end
    // };

    let tile_end: Vec<_> = parser(&params.tile_end.clone().unwrap_or(params.tile_start.clone()));

    let mut cells: Vec<(i32, i32, i32)> = (tile_start[0]..=tile_end[0])
        .map(|x| {
            (tile_start[1]..=tile_end[1])
                .zip(std::iter::repeat(x))
                .map(|(y, x)| (x, y, tile_start[2]))
        })
        .flatten()
        .collect();

    let mut result = Vec::new();
    let sys = System::new_all();
    let limit = sys.total_memory() / 4096;
    let mut total = cells.len();
    let mut current = 0;

    dbg!(&limit);
    println!("progress: 0%");
    while let Some(cell) = cells.pop() {
        let (index, geoms, count) = get_index(&manager, cell.0, cell.1, cell.2, limit as i64)
            .expect("Could not receive index");
        let ratio = count / limit as i64 + 1;
        dbg!(&geoms.len());
        let index = Arc::new(index);
        let geoms = Arc::new(geoms);
        let mut cells_inner = vec![cell];

        for level_i in cell.2..=params.level {
            let increase = level_i < params.level;

            let (cell_inside, cell_outside): (Vec<_>, Vec<_>) = cells_inner
                .par_iter()
                .map(|x| {
                    rayon::iter::repeat_n((x, increase as u32), 4_usize.pow(increase as u32))
                        .enumerate()
                        .map(|(i, (x, inc))| {
                            (
                                x.0 * 2_i32.pow(inc) + (i as i32) / 2,
                                x.1 * 2_i32.pow(inc) + (i as i32) % 2,
                                x.2 + inc as i32,
                            )
                        })
                })
                .flatten()
                .map(|point| {
                    let tile = st_tileenvelope(level_i as u32 + increase as u32, point.0, point.1);
                    let mut inter = index.locate_in_envelope_intersecting(&tile.envelope());
                    let mut counter = 0;
                    if count >= limit as i64 {
                        counter = index
                            .locate_in_envelope_intersecting(&tile.envelope())
                            .count() as i64;
                    }
                    let any_geom = inter.any(|x| geoms[x.data].intersects(&tile));
                    if !any_geom {
                        if geoms.len() as u64 == limit {
                            return (None, Some(point));
                        }
                        return (None, None);
                    }
                    if counter * ratio <= limit as i64 && geoms.len() as u64 == limit {
                        return (None, Some(point));
                    }
                    (Some(point), None)
                })
                .unzip();
            cells_inner = cell_inside.into_iter().flatten().collect();
            total = total + cell_outside.iter().flatten().count();
            cells.extend(cell_outside.iter().flatten());
            if level_i == params.level {
                result.extend(cells_inner.iter().map(|(x, y, _)| (*x, *y)));
            }
        }
        current += 1;
        println!("progress: {}%", current as f32 / total as f32);
    }

    //.map(|x| x.0.ok().zip(x.1.ok()).zip(x.2.okdraughtmapscorex| medle (draught, score, medt candidates: Vec<_> = cells
    //     .map(|(x, y)| {
    //         (
    //             x,
    //             y,
    //             stmt.query_row([params.level, x, y], |row| row.get::<_, i32>(0))
    //                 .unwrap(),
    //         )
    //     })
    //     .filter(|x| x.2 != 0)
    //     .map(|(x, y, _)| (x, y))
    //     .collect();
    Ok(result)
}

fn dist_normal(dist: f32) -> f32 {
    (1. - dist / 500.).clamp(0., 1.)
}

fn render_geom(
    point: CoordM,
    next_point: Option<CoordM>,
    dimensions: Option<DbDimensions>,
    tile: DbTile,
) -> (f32, f32) {
    let new_cell = Cell::from((tile.x, tile.y, tile.z as u32));
    let cell_iter = || std::iter::repeat_n(new_cell, 1);
    let conf = ErrorMeasurementConf::builder()
        .method(fafo::ErrorMeasurementMethod::Geodesic)
        .zoom(tile.z as u8)
        .build();
    if let Some(next_point) = next_point {
        let dist = conf
            .cell_distance_to_ground_truth((point.into(), next_point.into()), cell_iter())
            .iter()
            .map(|x| x.1)
            .last()
            .unwrap_or_default();
        let line = LineM::from((point, next_point));
        if let Some(dim) = dimensions {
            let (tri1, tri2) = line_to_triangle_pair(
                &line,
                dim.a as f64,
                dim.b as f64,
                dim.c as f64,
                dim.d as f64,
            );
            let cov = cells_relative_coverage_by_polygon((&tri1, &tri2), cell_iter())
                .last()
                .map(|x| x.1)
                .unwrap_or_default();
            return (cov as f32, dist_normal(dist as f32));
        }
        let cov = line_error_relative_to_perfect_and_centroid(
            (point.into(), next_point.into()),
            cell_iter(),
        )
        .iter()
        .map(|x| x.1)
        .last()
        .unwrap_or_default();
        return (cov as f32, dist_normal(dist as f32));
    }
    let dist = ground_truth_to_cell_centroid_geodesic(PointM::from(point), &new_cell);
    return (0., dist_normal(dist as f32));
}

fn score_cell(params: [f32; 6]) -> f32 {
    let weights = judweight_vessel();
    mul_arr_sum(params, weights)
}

fn mul_arr_sum<const N: usize>(a: [f32; N], b: [f32; N]) -> f32 {
    a.iter().zip(b.iter()).map(|(&a, &b)| a * b).sum()
}
fn render_cell_to_table(
    con: &Connection,
    cells: &[(i32, i32)],
    params: &RenderCell,
) -> Result<(), Box<dyn std::error::Error>> {
    let con = Arc::new(Mutex::new(
        con.try_clone().expect("Could not clone connection"),
    ));

    let sys = System::new_all();
    let thread_count = sys.cpus().len();

    // let chunks_size = std::cmp::max(cells.len() / (thread_count * 16), 2048);

    let result: Vec<_> = cells
        .par_iter()
        .map(|(x, y)| {
            let con = con.lock().unwrap().try_clone().unwrap();
            let data = request_cell(&con, *x, *y, params.level, params.level - 5);
            let params_s: Vec<_> = data
                .iter()
                .map(|(mmsi, ship_type, draught, _, _, _)| {
                    (
                        get_draught_dist_mmsi_normal(&con, *mmsi, *draught),
                        get_draught_dist_vessel_type_normal(&con, *mmsi, *draught),
                        get_draught_nulls_by_ship_type(&con, ship_type),
                        get_linear_regression(&con, ship_type),
                    )
                })
                .map(|x| {
                    x.0.zip(x.1).zip(x.2).zip(x.3).map(
                        |(((dist_mmsi, dist_vessel), nulls), r_sq)| {
                            (dist_mmsi, dist_vessel, nulls, r_sq)
                        },
                    )
                })
                .collect();
            let mut draught_score: Vec<_> = data
                .iter()
                .map(|(_, _, draught, point, next_point, dims)| {
                    (
                        draught,
                        render_geom(
                            *point,
                            *next_point,
                            *dims,
                            DbTile {
                                x: *x,
                                y: *y,
                                z: params.level,
                            },
                        ),
                    )
                })
                .zip(params_s.iter())
                .map(|((draught, (cov, dist)), param)| match param {
                    Some(p) => (*draught, score_cell([p.1, p.2, dist, cov, p.0, p.3])),
                    None => (*draught, 0.),
                })
                .collect();
            draught_score.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap().reverse());

            let result = draught_score
                .iter()
                .enumerate()
                .map(|(i, left)| {
                    draught_score[i..draught_score.len()]
                        .iter()
                        .map(|right| (left.0, gravity_model(left.1, left.0, right.1, right.0)))
                })
                .flatten()
                .find(|x| x.1 >= params.threshold)
                .unwrap_or_default();

            result
        })
        .collect();

    // con.execute_batch(
    //     "LOAD spatial; SET geometry_always_xy = true;
    //     CREATE OR REPLACE TABLE cand_cells (
    //         id BIGINT,
    //         x INTEGER,
    //         y INTEGER,
    //         z INTEGER
    //     );
    //     CREATE OR REPLACE TABLE cand_cell_relation (
    //         cell_id BIGINT,
    //         geom_id BIGINT
    //     );
    //     ",
    // )?;

    // let mut cand_cell_app = con.appender_with_columns("cand_cells", &["id", "x", "y", "z"])?;
    // let mut relation_app =
    //     con.appender_with_columns("cand_cell_relation", &["cell_id", "geom_id"])?;

    // cells
    //     .iter()
    //     .enumerate()
    //     .for_each(|(cell_id, (cells, geom_ids))| {
    //         cand_cell_app
    //             .append_row(params![cell_id as i64 + 1, cells.0, cells.1, params.level])
    //             .expect("Could not append cand cell");
    //         geom_ids.iter().for_each(|id| {
    //             relation_app
    //                 .append_row([cell_id as i64, *id])
    //                 .expect("Could not append relation ids")
    //         });
    //     });

    // panic!("Done");

    //     let sql = "WITH
    //   scored AS MATERIALIZED (
    //     SELECT
    //       draught,
    //       render_geom (
    //         point,
    //         next_point,
    //         dimensions,
    //         {'x': ?, 'y': ?, 'level': ?},
    //         parameters
    //       ) as score,
    //       median_draught
    //     FROM
    //       lines_with_geom a
    //     WHERE
    //       draught IS NOT NULL AND ST_Transform(ST_TileEnvelope (?, ?, ?), 'EPSG:3857', 'EPSG:4326') && a.geom
    //   )
    // SELECT
    //   a.draught::float as draught,
    //   combine_cell (
    //     a.draught::float,
    //     a.score,
    //     a.median_draught::float,
    //     b.draught::float,
    //     b.score,
    //     b.median_draught::float
    //   ) as reliability
    // FROM
    //   scored a
    //   LEFT JOIN scored b ON a.draught >= b.draught
    // WHERE reliability >= 0.53
    // ORDER BY draught, reliability DESC
    // LIMIT 1;";

    //     let chunk_size = cmp::max(cells.len() / 16, 2048);
    //     let result: Vec<_> = cells
    //         .par_chunks(chunk_size)
    //         .map(|x| (x, con.lock().unwrap().try_clone().unwrap()))
    //         .map(|(cells, con)| {
    //             let mut stmt = con.prepare(sql).expect("Could not prepare statement");
    //             cells
    //                 .iter()
    //                 .map(|(x, y)| {
    //                     stmt.query_one(
    //                         params![
    //                             *x as u32,
    //                             *y as u32,
    //                             params.level as u8,
    //                             params.level,
    //                             *x,
    //                             *y
    //                         ],
    //                         |x| Ok((x.get::<_, f32>(0), x.get::<_, f32>(1))),
    //                     )
    //                 })
    //                 .collect::<Vec<_>>()
    //         })
    //         .flatten()
    //         .map(|x| x.ok())
    //         .map(|x| x.map(|x| (x.0.unwrap_or_default(), x.1.unwrap_or_default())))
    //         .map(|x| x.unwrap_or_default())
    //         .collect();

    // Write cells to table
    let con = con.lock().unwrap().try_clone().unwrap();
    con.execute_batch(
        "
            CREATE OR REPLACE TABLE render.render (
                    x INTEGER,
                    y INTEGER,
                    z INTEGER,
                    draught FLOAT,
                    reliability FLOAT
                );
            ",
    )?;

    let mut app = con.appender_to_db("render", "render")?;
    let result: Result<Vec<_>, _> = cells
        .iter()
        .zip(result.iter())
        .map(|((x, y), (draught, rely))| {
            app.append_row(params![*x, *y, params.level, *draught, *rely])
        })
        .collect();
    result?;
    Ok(())
}
