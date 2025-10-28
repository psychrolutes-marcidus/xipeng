 WITH discrete_status AS (
         SELECT DISTINCT vd.mmsi,
            make_timestamptz(dd.year_no::integer, dd.month_no::integer, dd.day_no::integer, td.hour_no::integer, td.minute_no::integer, td.second_no::double precision) AS "timestamp",
            ansd.status_name
           FROM fact.ais_point_fact apf,
            dim.ais_nav_status_dim ansd,
            dim.vessel_dim vd,
            dim.date_dim dd,
            dim.time_dim td
          WHERE apf.vessel_id = vd.vessel_id AND apf.ais_nav_status_id = ansd.ais_nav_status_id AND apf.date_id = dd.date_id AND apf.time_id = td.time_id
          ORDER BY vd.mmsi, (make_timestamptz(dd.year_no::integer, dd.month_no::integer, dd.day_no::integer, td.hour_no::integer, td.minute_no::integer, td.second_no::double precision))
        ), grouped_status AS (
         SELECT discrete_status.mmsi,
            discrete_status."timestamp",
            discrete_status.status_name,
            row_number() OVER (PARTITION BY discrete_status.mmsi ORDER BY discrete_status.mmsi, discrete_status."timestamp") AS mmsi_part,
            row_number() OVER (PARTITION BY discrete_status.mmsi, discrete_status.status_name ORDER BY discrete_status.mmsi, discrete_status."timestamp") AS mmsi_status_part,
            row_number() OVER (PARTITION BY discrete_status.mmsi ORDER BY discrete_status.mmsi, discrete_status."timestamp") - row_number() OVER (PARTITION BY discrete_status.mmsi, discrete_status.status_name ORDER BY discrete_status.mmsi, discrete_status."timestamp") AS seq
           FROM discrete_status
          ORDER BY discrete_status.mmsi, discrete_status."timestamp"
        )
 SELECT mmsi,
    status_name,
    min("timestamp") AS time_begin,
    max("timestamp") AS time_end
   FROM grouped_status
  WHERE status_name::text = 'aground'::text OR status_name::text = 'ais-sart (active)'::text OR status_name::text = 'at anchor'::text OR status_name::text = 'constrained by her draught'::text OR status_name::text = 'engaged in fishing'::text OR status_name::text = 'moored'::text OR status_name::text = 'not under command'::text OR status_name::text = 'restricted maneuverability'::text OR status_name::text = 'under way sailing'::text OR status_name::text = 'under way using engine'::text
  GROUP BY mmsi, status_name, seq
  ORDER BY mmsi, (min("timestamp"));