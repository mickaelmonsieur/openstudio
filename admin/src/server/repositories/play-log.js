export async function countPlayLog(db) {
  const { rows } = await db.query('SELECT COUNT(*)::integer AS total FROM play_log');
  return rows[0].total;
}

export async function listPlayLog(db, { limit, offset }) {
  const { rows } = await db.query(
    `
    SELECT
      p.id,
      to_char(p.played_at, 'YYYY-MM-DD HH24:MI:SS') AS played_at,
      s.name                                          AS station,
      a.name                                          AS artist,
      t.title                                         AS title,
      ROUND(p.played_duration::numeric, 1)            AS played_duration
    FROM play_log p
    LEFT JOIN stations s ON s.id = p.station_id
    LEFT JOIN tracks   t ON t.id = p.track_id
    LEFT JOIN artists  a ON a.id = t.artist_id
    ORDER BY p.played_at DESC
    LIMIT $1 OFFSET $2
    `,
    [limit, offset]
  );

  return rows;
}
