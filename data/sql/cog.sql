 SELECT vd.mmsi,
    make_timestamptz(dd.year_no::integer, dd.month_no::integer, dd.day_no::integer, td.hour_no::integer, td.minute_no::integer, td.second_no::double precision) AS "timestamp",
    apf.cog::real AS cog
   FROM dim.vessel_dim vd,
    fact.ais_point_fact apf,
    dim.date_dim dd,
    dim.time_dim td
  WHERE apf.vessel_id = vd.vessel_id AND td.time_id = apf.time_id AND dd.date_id = apf.date_id AND apf.cog IS NOT NULL;