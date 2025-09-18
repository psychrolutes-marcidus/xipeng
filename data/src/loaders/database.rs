use crate::errors::DatabaseError;
use crate::tables::*;
use chrono::prelude::*;
use postgres::{Client, Config, NoTls};
use wkb::reader::read_wkb;
use linesonmaps::types::linestringm::LineStringM;

pub struct DbConn {
    pub conn: Client,
}

impl DbConn {
    pub fn new() -> Result<Self, DatabaseError> {
        let hostname: String = std::env::var("DB_HOSTNAME")?;
        let port_str: String = std::env::var("DB_PORT")?;
        let port_i: u16 = port_str.parse::<u16>()?;
        let user: String = std::env::var("DB_USERNAME")?;
        let password: String = std::env::var("DB_PASSWORD")?;
        let db_name: String = std::env::var("DB_NAME")?;

        let mut db_conf = Config::new();

        db_conf.host(&hostname);
        db_conf.port(port_i);
        db_conf.user(&user);
        db_conf.password(password);
        db_conf.dbname(&db_name);

        let client = db_conf
            .connect(NoTls)
            .map_err(|e| DatabaseError::Connect(e))?;
        Ok(Self { conn: client })
    }

    pub fn fetch_data(
        &mut self,
        time_begin: NaiveDateTime,
        time_end: NaiveDateTime,
    ) -> Result<Ships, DatabaseError> {
        let nav = fetch_nav_status(&mut self.conn, time_begin, time_end);
        let draught = fetch_draught(&mut self.conn, time_begin, time_end);
        todo!()
    }
}

fn fetch_nav_status(
    conn: &mut Client,
    time_begin: NaiveDateTime,
    time_end: NaiveDateTime,
) -> Result<nav_status::NavStatus, DatabaseError> {
    let mut nav_status_table: nav_status::NavStatus = nav_status::NavStatus::new();
    let result = conn
        .query(
            "SELECT mmsi, time_begin, time_end, status_name
                FROM PROGRAM_DATA.NAV_STATUS
                WHERE
                    time_end >= $1 AND time_begin <= $2",
            &[&time_begin, &time_end],
        )
        .map_err(|e| DatabaseError::QueryError(e))?;

    let size = result.len();
    nav_status_table.mmsi.reserve(size);
    nav_status_table.time_begin.reserve(size);
    nav_status_table.time_end.reserve(size);
    nav_status_table.nav_status.reserve(size);

    for row in result {
        let mmsi: i64 = row.get("mmsi");
        let time_begin: NaiveDateTime = row.get("time_begin");
        let time_end: NaiveDateTime = row.get("time_end");
        let status: String = row.get("status_name");

        let status_parsed = nav_status::nav_status_converter(&status);

        if status_parsed.is_some() {
            nav_status_table.mmsi.push(mmsi as u64);
            nav_status_table.time_begin.push(time_begin);
            nav_status_table.time_end.push(time_end);
            nav_status_table.nav_status.push(status_parsed.unwrap());
        }
    }

    Ok(nav_status_table)
}

fn fetch_draught(
    conn: &mut Client,
    time_begin: NaiveDateTime,
    time_end: NaiveDateTime,
) -> Result<ship_draught::Draught, DatabaseError> {
    let mut draught_table: ship_draught::Draught = ship_draught::Draught::new();

    let result = conn
        .query(
            "SELECT mmsi, time_begin, time_end, draught
                            FROM
                                PROGRAM_DATA.DRAUGHT
                            WHERE
                                TIME_END >= $1 AND TIME_BEGIN <= $2",
            &[&time_begin, &time_end],
        )
        .map_err(|e| DatabaseError::QueryError(e))?;

    let size = result.len();

    draught_table.mmsi.reserve(size);
    draught_table.time_begin.reserve(size);
    draught_table.time_end.reserve(size);
    draught_table.draught.reserve(size);

    for row in result {
        let mmsi: i64 = row.get("mmsi");
        let time_begin: NaiveDateTime = row.get("time_begin");
        let time_end: NaiveDateTime = row.get("time_end");
        let draught: f32 = row.get("draught");

        draught_table.mmsi.push(mmsi as u64);
        draught_table.time_begin.push(time_begin);
        draught_table.time_end.push(time_end);
        draught_table.draught.push(draught);
    }

    Ok(draught_table)
}

fn fetch_cog(
    conn: &mut Client,
    time_begin: NaiveDateTime,
    time_end: NaiveDateTime,
) -> Result<cog::Cog, DatabaseError> {
    let mut cog_table: cog::Cog = cog::Cog::new();

    let result = conn
        .query(
            "SELECT mmsi, timestamp, cog
            FROM	PROGRAM_DATA.COG
            WHERE timestamp >= $1 AND timestamp <= $2",
            &[&time_begin, &time_end],
        )
        .map_err(|e| DatabaseError::QueryError(e))?;

    let size = result.len();

    cog_table.mmsi.reserve(size);
    cog_table.time.reserve(size);
    cog_table.cog.reserve(size);

    for row in result {
        let mmsi: i64 = row.get("mmsi");
        let time: NaiveDateTime = row.get("timestamp");
        let cog: f32 = row.get("cog");
        cog_table.mmsi.push(mmsi as u64);
        cog_table.time.push(time);
        cog_table.cog.push(cog);
    }

    Ok(cog_table)
}

fn fetch_sog(
    conn: &mut Client,
    time_begin: NaiveDateTime,
    time_end: NaiveDateTime,
) -> Result<sog::Sog, DatabaseError> {
    let mut sog_table: sog::Sog = sog::Sog::new();

    let result = conn
        .query(
            "SELECT mmsi, timestamp, sog
        FROM PROGRAM_DATA.SOG
        WHERE timestamp >= $1 AND timestamp <= $2",
            &[&time_begin, &time_end],
        )
        .map_err(|e| DatabaseError::QueryError(e))?;

    let size = result.len();

    sog_table.mmsi.reserve(size);
    sog_table.time.reserve(size);
    sog_table.sog.reserve(size);

    for row in result {
        let mmsi: i64 = row.get("mmsi");
        let time: NaiveDateTime = row.get("timestamp");
        let sog: f32 = row.get("sog");
        sog_table.mmsi.push(mmsi as u64);
        sog_table.time.push(time);
        sog_table.sog.push(sog);
    }
    Ok(sog_table)
}

fn fetch_rot(
    conn: &mut Client,
    time_begin: NaiveDateTime,
    time_end: NaiveDateTime,
) -> Result<rot::Rot, DatabaseError> {
    let mut rot_table: rot::Rot = rot::Rot::new();

    let result = conn
        .query(
            "SELECT mmsi, timestamp, rot
    FROM PROGRAM_DATA.ROT
    WHERE timestamp >= $1 AND timestamp <= $2",
            &[&time_begin, &time_end],
        )
        .map_err(|e| DatabaseError::QueryError(e))?;

    let size = result.len();

    rot_table.mmsi.reserve(size);
    rot_table.time.reserve(size);
    rot_table.rot.reserve(size);

    for row in result {
        let mmsi: i64 = row.get("mmsi");
        let time: NaiveDateTime = row.get("timestamp");
        let rot: f32 = row.get("rot");
        rot_table.mmsi.push(mmsi as u64);
        rot_table.time.push(time);
        rot_table.rot.push(rot);
    }

    Ok(rot_table)
}

fn fetch_trajectories(
    conn: &mut Client,
    time_begin: NaiveDateTime,
    time_end: NaiveDateTime,
) -> Result<trajectories::Trajectories, DatabaseError> {
    let mut trajectories_table = trajectories::Trajectories::new();

    let result = conn
        .query(
            "SELECT mmsi, ST_AsBinary(ST_FilterByM(traj, $1, $2, true), 'NDR') as traj
FROM PROGRAM_DATA.trajectories
WHERE ST_IsEmpty(ST_FilterByM(traj, $1, $2) = false
LIMIT 100",
            &[
                &(time_begin.and_utc().timestamp() as f64),
                &(time_end.and_utc().timestamp() as f64),
            ],
        )
        .map_err(|e| DatabaseError::QueryError(e))?;

    let size = result.len();

    trajectories_table.mmsi.reserve(size);
    trajectories_table.trajectory.reserve(size);

    for row in result {
        let mmsi: i64 = row.get("mmsi");
        let traj: Vec<u8> = row.get("traj");

        let wkb_data = read_wkb(traj.as_slice())?;
        let lsm = LineStringM::try_from(wkb_data)?;

        trajectories_table.mmsi.push(mmsi as u64);
        trajectories_table.trajectory.push(traj);
    }

    todo!()
}
