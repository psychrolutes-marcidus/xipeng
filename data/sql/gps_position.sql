 WITH duplicate_count AS (
         SELECT DISTINCT v.mmsi,
            conf.to_bow AS a,
            conf.to_stern AS b,
            conf.to_port AS c,
            conf.to_starboard AS d,
            count(*) AS count
           FROM dim.vessel_dim v,
            fact.ais_point_fact f,
            dim.vessel_config_dim conf
          WHERE v.vessel_id = f.vessel_id AND f.vessel_config_id = conf.vessel_config_id AND conf.to_bow IS NOT NULL AND conf.to_stern IS NOT NULL AND conf.to_port IS NOT NULL AND conf.to_starboard IS NOT NULL
          GROUP BY v.mmsi, conf.to_bow, conf.to_stern, conf.to_port, conf.to_starboard
          ORDER BY (count(*)) DESC
        )
 SELECT DISTINCT ON (mmsi) mmsi,
    a,
    b,
    c,
    d
   FROM duplicate_count dc;