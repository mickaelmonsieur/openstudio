const LIST_COLUMNS = `
  t.id,
  t.artist_id,
  t.genre_id,
  t.title,
  t.album,
  t.year,
  t.duration,
  t.sample_rate,
  t.cue_in,
  t.intro,
  t.hook_in,
  t.hook_out,
  t.loop_in,
  t.loop_out,
  t.outro,
  t.cue_out,
  t.path,
  t.active,
  t.subcategory_id,
  a.name  AS artist,
  g.name  AS genre,
  sc.name AS subcategory,
  c.name  AS category
`;

const FROM_JOIN = `
  FROM tracks t
  LEFT JOIN artists       a  ON a.id  = t.artist_id
  LEFT JOIN genres        g  ON g.id  = t.genre_id
  LEFT JOIN subcategories sc ON sc.id = t.subcategory_id
  LEFT JOIN categories    c  ON c.id  = sc.category_id
`;

export async function countTracks(db, search = '', categoryId = null) {
  const { where, values } = buildSearchWhere(search, categoryId);
  const { rows } = await db.query(
    `
    SELECT COUNT(*)::integer AS total
    ${FROM_JOIN}
    ${where}
    `,
    values
  );
  return rows[0].total;
}

export async function listTracks(db, { limit, offset, search = '', categoryId = null }) {
  const { where, values } = buildSearchWhere(search, categoryId);
  const limitParam = values.length + 1;
  const offsetParam = values.length + 2;

  const { rows } = await db.query(
    `
    SELECT ${LIST_COLUMNS}
    ${FROM_JOIN}
    ${where}
    ORDER BY a.name NULLS LAST, t.title
    LIMIT $${limitParam} OFFSET $${offsetParam}
    `,
    [...values, limit, offset]
  );

  return rows;
}

function buildSearchWhere(search, categoryId = null) {
  const terms = String(search || '').trim().split(/\s+/).filter(Boolean).slice(0, 8);
  const clauses = [];
  const values = [];

  for (const term of terms) {
    values.push(`%${escapeLike(term)}%`);
    const param = `$${values.length}`;
    clauses.push(`
      (
        t.id::text ILIKE ${param} ESCAPE '\\'
        OR COALESCE(a.name, '') ILIKE ${param} ESCAPE '\\'
        OR COALESCE(t.title, '') ILIKE ${param} ESCAPE '\\'
        OR COALESCE(t.album, '') ILIKE ${param} ESCAPE '\\'
        OR COALESCE(g.name, '') ILIKE ${param} ESCAPE '\\'
        OR COALESCE(c.name, '') ILIKE ${param} ESCAPE '\\'
        OR COALESCE(sc.name, '') ILIKE ${param} ESCAPE '\\'
        OR COALESCE(t.year::text, '') ILIKE ${param} ESCAPE '\\'
      )
    `);
  }

  if (categoryId) {
    values.push(categoryId);
    clauses.push(`sc.category_id = $${values.length}`);
  }

  return {
    where: clauses.length > 0 ? `WHERE ${clauses.join(' AND ')}` : '',
    values
  };
}

function escapeLike(value) {
  return String(value).replace(/[\\%_]/g, '\\$&');
}

export async function getTrack(db, id) {
  const { rows } = await db.query(
    `
    SELECT ${LIST_COLUMNS}
    ${FROM_JOIN}
    WHERE t.id = $1
    `,
    [id]
  );

  return rows[0] || null;
}

export async function createTrack(db, data) {
  const { rows } = await db.query(
    `
    INSERT INTO tracks (
      artist_id,
      genre_id,
      title,
      album,
      year,
      duration,
      sample_rate,
      path,
      subcategory_id,
      active
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
    RETURNING id
    `,
    [
      data.artist_id || null,
      data.genre_id,
      data.title,
      data.album,
      data.year || null,
      data.duration || 0,
      data.sample_rate || 44100,
      data.path,
      data.subcategory_id || null,
      data.active
    ]
  );

  return getTrack(db, rows[0].id);
}

export async function trackExistsByPath(db, trackPath) {
  const { rows } = await db.query(
    `
    SELECT id
    FROM tracks
    WHERE path = $1
    LIMIT 1
    `,
    [trackPath]
  );

  return rows[0] || null;
}

export async function updateTrack(db, id, data) {
  const { rowCount } = await db.query(
    `
    UPDATE tracks
    SET artist_id      = $2,
        genre_id       = $3,
        title          = $4,
        album          = $5,
        year           = $6,
        subcategory_id = $7,
        active         = $8
    WHERE id = $1
    `,
    [id, data.artist_id, data.genre_id, data.title, data.album, data.year || null, data.subcategory_id || null, data.active]
  );

  return rowCount > 0;
}

export async function updateTrackCuePoint(db, id, field, value) {
  const column = CUE_POINT_COLUMNS[field];
  if (!column) return null;

  const { rows } = await db.query(
    `
    UPDATE tracks
    SET ${column} = $2
    WHERE id = $1
    RETURNING ${cuePointSelectList()}
    `,
    [id, value]
  );

  return rows[0] || null;
}

export function cuePointSelectList() {
  return Object.values(CUE_POINT_COLUMNS).join(', ');
}

const CUE_POINT_COLUMNS = {
  cue_in: 'cue_in',
  intro: 'intro',
  hook_in: 'hook_in',
  hook_out: 'hook_out',
  loop_in: 'loop_in',
  loop_out: 'loop_out',
  outro: 'outro',
  cue_out: 'cue_out'
};

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

export async function listGenres(db) {
  const { rows } = await db.query(`
    SELECT id, name
    FROM genres
    ORDER BY name
  `);

  return rows;
}

export async function findGenreByName(db, name) {
  const normalizedName = String(name || '').trim();
  if (!normalizedName) return null;

  const { rows } = await db.query(
    `
    SELECT id, name
    FROM genres
    WHERE lower(name) = lower($1)
    LIMIT 1
    `,
    [normalizedName]
  );

  return rows[0] || null;
}
