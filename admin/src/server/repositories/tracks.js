const LIST_COLUMNS = `
  t.id,
  t.artist_id,
  t.title,
  t.album,
  t.year,
  t.duration,
  t.active,
  t.subcategory_id,
  a.name  AS artist,
  sc.name AS subcategory,
  c.name  AS category
`;

const FROM_JOIN = `
  FROM tracks t
  LEFT JOIN artists       a  ON a.id  = t.artist_id
  LEFT JOIN subcategories sc ON sc.id = t.subcategory_id
  LEFT JOIN categories    c  ON c.id  = sc.category_id
`;

export async function countTracks(db) {
  const { rows } = await db.query('SELECT COUNT(*)::integer AS total FROM tracks');
  return rows[0].total;
}

export async function listTracks(db, { limit, offset }) {
  const { rows } = await db.query(
    `
    SELECT ${LIST_COLUMNS}
    ${FROM_JOIN}
    ORDER BY a.name NULLS LAST, t.title
    LIMIT $1 OFFSET $2
    `,
    [limit, offset]
  );

  return rows;
}

export async function updateTrack(db, id, data) {
  const { rowCount } = await db.query(
    `
    UPDATE tracks
    SET artist_id      = $2,
        title          = $3,
        album          = $4,
        year           = $5,
        subcategory_id = $6,
        active         = $7
    WHERE id = $1
    `,
    [id, data.artist_id, data.title, data.album, data.year || null, data.subcategory_id || null, data.active]
  );

  return rowCount > 0;
}

export async function hasScheduledQueue(db, id) {
  const { rows } = await db.query(
    `
    SELECT COUNT(*)::integer AS cnt
    FROM queue
    WHERE track_id   = $1
      AND played     = FALSE
      AND scheduled_at > NOW()
    `,
    [id]
  );

  return rows[0].cnt > 0;
}

export async function deleteTrack(db, id) {
  const { rowCount } = await db.query(
    'DELETE FROM tracks WHERE id = $1',
    [id]
  );

  return rowCount > 0;
}

export async function listSubcategoriesWithCategory(db) {
  const { rows } = await db.query(`
    SELECT
      sc.id,
      sc.name,
      c.name AS category_name
    FROM subcategories sc
    JOIN categories c ON c.id = sc.category_id
    ORDER BY c.name, sc.name
  `);

  return rows;
}
