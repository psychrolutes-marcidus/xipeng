 WITH discrete_draught AS (
         SELECT DISTINCT vd.mmsi,
            make_timestamptz(dd.year_no::integer, dd.month_no::integer, dd.day_no::integer, td.hour_no::integer, td.minute_no::integer, td.second_no::double precision) AS "timestamp",
            apf.draught
           FROM fact.ais_point_fact apf,
            dim.vessel_dim vd,
            dim.date_dim dd,
            dim.time_dim td
          WHERE apf.vessel_id = vd.vessel_id AND apf.date_id = dd.date_id AND apf.time_id = td.time_id
          ORDER BY vd.mmsi, (make_timestamptz(dd.year_no::integer, dd.month_no::integer, dd.day_no::integer, td.hour_no::integer, td.minute_no::integer, td.second_no::double precision))
        ), grouped_draught AS (
         SELECT discrete_draught.mmsi,
            discrete_draught."timestamp",
            discrete_draught.draught,
            row_number() OVER (PARTITION BY discrete_draught.mmsi ORDER BY discrete_draught.mmsi, discrete_draught."timestamp") AS mmsi_part,
            row_number() OVER (PARTITION BY discrete_draught.mmsi, discrete_draught.draught ORDER BY discrete_draught.mmsi, discrete_draught."timestamp") AS mmsi_draught_part,
            row_number() OVER (PARTITION BY discrete_draught.mmsi ORDER BY discrete_draught.mmsi, discrete_draught."timestamp") - row_number() OVER (PARTITION BY discrete_draught.mmsi, discrete_draught.draught ORDER BY discrete_draught.mmsi, discrete_draught."timestamp") AS seq
           FROM discrete_draught
          ORDER BY discrete_draught.mmsi, discrete_draught."timestamp"
        )
 SELECT mmsi,
    draught::real AS draught,
    min("timestamp") AS time_begin,
    max("timestamp") AS time_end
   FROM grouped_draught
  WHERE draught IS NOT NULL
  GROUP BY mmsi, draught, seq
  ORDER BY mmsi, (min("timestamp"));