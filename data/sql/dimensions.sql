 WITH duplicate_count AS (
         SELECT DISTINCT v.mmsi,
            conf.width,
            conf.length,
            count(*) AS count
           FROM dim.vessel_dim v,
            fact.ais_point_fact f,
            dim.vessel_config_dim conf
          WHERE v.vessel_id = f.vessel_id AND f.vessel_config_id = conf.vessel_config_id AND conf.width IS NOT NULL AND conf.length IS NOT NULL
          GROUP BY v.mmsi, conf.width, conf.length
          ORDER BY (count(*)) DESC
        )
 SELECT DISTINCT ON (mmsi) mmsi,
    width,
    length
   FROM duplicate_count dc;