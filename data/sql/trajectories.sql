 SELECT v.mmsi,
    st_makeline(array_agg(DISTINCT st_pointm(st_x(ais.geom::geometry), st_y(ais.geom::geometry), EXTRACT(epoch FROM make_timestamptz(dat.year_no::integer, dat.month_no::integer, dat.day_no::integer, tim.hour_no::integer, tim.minute_no::integer, tim.second_no::double precision))::double precision, 4326))) AS traj
   FROM fact.ais_point_fact ais
     JOIN dim.vessel_dim v ON ais.vessel_id = v.vessel_id
     JOIN dim.time_dim tim ON tim.time_id = ais.time_id
     JOIN dim.date_dim dat ON dat.date_id = ais.date_id
  WHERE ais.lat <> 91::double precision
  GROUP BY v.mmsi;